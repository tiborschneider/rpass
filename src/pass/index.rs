// rpass: a password manager based on pass, written in rust
// Copyright (C) 2020, Tibor Schneider
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see http://www.gnu.org/licenses/

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::panic;
use std::process::{Command, Stdio};
use std::time::SystemTime;

use petgraph::graph::{Graph, NodeIndex};

use uuid::Uuid;

use crate::Loading;
use crate::config::CFG;
use crate::errors::{Error, Result};

thread_local! {
    pub static INDEX: RefCell<Index> = RefCell::new(Index::default());
}

#[derive(Debug)]
pub struct Index {
    timestamp: SystemTime,
    index: Vec<(Uuid, String)>,
}

pub fn get_index() -> Result<Vec<(Uuid, String)>> {
    INDEX.with(|index| {
        if index.borrow().is_depricated()? {
            index.replace(Index::read()?);
        }
        Ok(index.borrow().index.clone())
    })
}

impl Default for Index {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::UNIX_EPOCH,
            index: Vec::new(),
        }
    }
}

impl Index {
    fn read() -> Result<Self> {
        Ok(Self {
            timestamp: Index::current_timestamp()?,
            index: read_index()?,
        })
    }

    fn is_depricated(&self) -> Result<bool> {
        let timestamp = Index::current_timestamp()?;
        Ok(self.timestamp < timestamp)
    }

    fn current_timestamp() -> Result<SystemTime> {
        let mut index_file = home::home_dir().unwrap();
        index_file.push(".password-store/uuids/index.gpg");
        let metadata = std::fs::metadata(&index_file)?;
        let timestamp = metadata.modified()?;
        Ok(timestamp)
    }
}

#[allow(dead_code)]
fn read_index() -> Result<Vec<(Uuid, String)>> {
    let _loading = Loading::new("Reading the index...")?;

    // execute pass command
    let output = Command::new("pass")
        .arg(format!("{}/{}", CFG.main.uuid_folder, CFG.main.index_entry))
        .output()?;

    if !output.status.success() {
        return Err(Error::NoIndexFile);
    }

    let result_utf8 = output.stdout;

    // parse to str
    let result = String::from_utf8(result_utf8)?;

    // generate the resulting vector

    let index_list: std::result::Result<Vec<(Uuid, String)>, _> = panic::catch_unwind(|| {
        result
            .lines()
            .map(|s| s.split(' ').collect())
            .map(|v: Vec<&str>| (Uuid::parse_str(v[0]).unwrap(), v[1].to_string()))
            .collect()
    });

    match index_list {
        Ok(l) => Ok(l),
        Err(_) => Err(Error::Other("UUID Error: cannot parse uuid!".to_string())),
    }
}

pub fn to_hashmap<'a>(index_list: &'a [(Uuid, String)]) -> HashMap<Uuid, &'a str> {
    let mut map: HashMap<Uuid, &'a str> = HashMap::new();
    for (id, path) in index_list {
        map.insert(*id, path);
    }
    map
}

pub fn to_hashmap_reverse<'a>(index_list: &'a [(Uuid, String)]) -> HashMap<&'a str, Uuid> {
    let mut map: HashMap<&'a str, Uuid> = HashMap::new();
    for (id, path) in index_list {
        map.insert(path, *id);
    }
    map
}

pub fn to_graph<'a>(index_list: &'a [(Uuid, String)]) -> (Graph<&'a str, ()>, NodeIndex) {
    let mut g: Graph<&'a str, ()> = Graph::new();

    let root = g.add_node("root");
    let mut cur_path: Vec<NodeIndex> = Vec::new();

    let mut index_iter = index_list.iter();

    // handle first element
    let (_, full_path) = index_iter.next().unwrap();
    for node in full_path.split('/') {
        cur_path.push(g.add_node(node));
        if cur_path.len() == 1 {
            g.add_edge(root, cur_path[0], ());
        } else {
            g.add_edge(
                cur_path[cur_path.len() - 2],
                cur_path[cur_path.len() - 1],
                (),
            );
        }
    }

    // handle all other elements
    for (_, full_path) in index_iter {
        let path: Vec<&str> = full_path.split('/').collect();

        // determine index where both are the same
        let same = path
            .iter()
            .zip(cur_path.iter().filter_map(|x| g.node_weight(*x)))
            .enumerate()
            .filter(|(_, (x, y))| x != y)
            .map(|(i, _)| i)
            .next()
            .unwrap_or(path.len().min(cur_path.len()));
        assert!(same <= cur_path.len());

        // remove all elements until last greatest common parent is reached
        while same < cur_path.len() {
            cur_path.pop();
        }

        // add all new elements
        for node in path[same..path.len()].iter() {
            cur_path.push(g.add_node(node));
            if cur_path.len() == 1 {
                g.add_edge(root, cur_path[0], ());
            } else {
                g.add_edge(
                    cur_path[cur_path.len() - 2],
                    cur_path[cur_path.len() - 1],
                    (),
                );
            }
        }
    }

    (g, root)
}

pub fn write(index_list: &[(Uuid, String)]) -> Result<()> {
    let mut p = Command::new("pass")
        .arg("insert")
        .arg("--multiline")
        .arg(format!("{}/{}", CFG.main.uuid_folder, CFG.main.index_entry))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;

    if let Some(mut writer) = p.stdin.take() {
        for (id, path) in index_list {
            writer.write_all(&format!("{} {}", id, path).into_bytes())?;
            writer.write_all("\n".as_bytes())?;
        }
    }

    p.wait()?;

    Ok(())
}

pub fn insert(id: Uuid, path: &str) -> Result<()> {
    let mut index_list = get_index()?;
    index_list.push((id, path.to_string()));
    write(&index_list)
}

pub fn remove(id: Uuid) -> Result<()> {
    let index_list: Vec<(Uuid, String)> = get_index()?.into_iter().filter(|x| x.0 != id).collect();

    // remove the pass entry
    Command::new("pass")
        .arg("rm")
        .arg("--force")
        .arg(format!("{}/{}", CFG.main.uuid_folder, id))
        .output()?;

    write(&index_list)
}

pub fn mv(id: Uuid, dst: String) -> Result<()> {
    let mut index_list: Vec<(Uuid, String)> =
        get_index()?.into_iter().filter(|x| x.0 != id).collect();

    index_list.push((id, dst));
    write(&index_list)
}
