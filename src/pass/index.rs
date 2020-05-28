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

use std::panic;
use std::io::{Error, ErrorKind, Write};
use std::process::{Command, Stdio};
use std::collections::HashMap;

use petgraph::graph::{Graph, NodeIndex};

use uuid::Uuid;

use crate::def;

#[allow(dead_code)]
pub fn get_index() -> Result<Vec<(Uuid, String)>, Error> {
    // execute pass command
    let output = Command::new("pass")
        .arg(format!("{}/{}", def::UUID_FOLDER, def::INDEX_ENTRY))
        .output()?;

    if !output.status.success() {
        return Err(Error::new(ErrorKind::NotFound, "Index file was not found!"));
    }

    let result_utf8 = output.stdout;

    // parse to str
    let result = match String::from_utf8(result_utf8) {
        Ok(r) => r,
        Err(_) => Err(Error::new(ErrorKind::InvalidData, "Cannot parse utf8!"))?
    };

    // generate the resulting vector

    let index_list: Result<Vec<(Uuid, String)>, _> = panic::catch_unwind(||
        result.lines()
            .map(|s| s.split(" ").collect())
            .map(|v: Vec<&str>| (Uuid::parse_str(v[0]).unwrap(), v[1].to_string()))
            .collect()
    );

    match index_list {
        Ok(l) => Ok(l),
        Err(_) => Err(Error::new(ErrorKind::InvalidData, "Invalid UUID"))
    }

}

pub fn to_hashmap<'a>(index_list: &'a Vec<(Uuid, String)>) -> HashMap<Uuid, &'a str> {
    let mut map: HashMap<Uuid, &'a str> = HashMap::new();
    for (id, path) in index_list {
        map.insert(id.clone(), path);
    }
    map
}

pub fn to_hashmap_reverse<'a>(index_list: &'a Vec<(Uuid, String)>) -> HashMap<&'a str, Uuid> {
    let mut map: HashMap<&'a str, Uuid> = HashMap::new();
    for (id, path) in index_list {
        map.insert(path, id.clone());
    }
    map
}

pub fn to_graph<'a>(index_list: &'a Vec<(Uuid, String)>) -> (Graph<&'a str, ()>, NodeIndex) {

    let mut g: Graph<&'a str, ()> = Graph::new();

    let root = g.add_node("root");
    let mut cur_path: Vec<NodeIndex> = Vec::new();

    let mut index_iter = index_list.into_iter();

    // handle first element
    let (_, full_path) = index_iter.next().unwrap();
    for node in full_path.split("/") {
        cur_path.push(g.add_node(node));
        if cur_path.len() == 1 {
            g.add_edge(root, cur_path[0], ());
        } else {
            g.add_edge(cur_path[cur_path.len() - 2], cur_path[cur_path.len() - 1], ());
        }
    }

    // handle all other elements
    for (_, full_path) in index_iter {
        let path: Vec<&str> = full_path.split("/").collect();

        // determine index where both are the same
        let min_idx = if cur_path.len() > path.len() { cur_path.len() } else { path.len() };
        let mut same = 0;
        for i in 0..min_idx { if g.node_weight(cur_path[i]).unwrap() != &path[i] { same = i; break; } }
        assert!(same < cur_path.len());

        // remove all elements until last greatest common parent is reached
        while same < cur_path.len() { cur_path.pop(); };

        // add all new elements
        for node in path[same..path.len()].into_iter() {
            cur_path.push(g.add_node(node));
            if cur_path.len() == 1 {
                g.add_edge(root, cur_path[0], ());
            } else {
                g.add_edge(cur_path[cur_path.len() - 2], cur_path[cur_path.len() - 1], ());
            }
        }
    }

    (g, root)

}

pub fn write(index_list: &Vec<(Uuid, String)>) -> Result<(), Error> {

    let mut p = Command::new("pass")
        .arg("insert")
        .arg("--multiline")
        .arg(format!("{}/{}", def::UUID_FOLDER, def::INDEX_ENTRY))
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

pub fn insert(id: Uuid, path: &String) -> Result<(), Error> {
    let mut index_list = get_index()?;
    index_list.push((id, path.to_string()));
    write(&index_list)
}

pub fn remove(id: Uuid) -> Result<(), Error> {

    let index_list: Vec<(Uuid, String)> = get_index()?
        .into_iter()
        .filter(|x| x.0 != id)
        .collect();

    // remove the pass entry
    Command::new("pass")
        .arg("rm")
        .arg("--force")
        .arg(format!("{}/{}", def::UUID_FOLDER, id))
        .output()?;

    write(&index_list)

}

pub fn mv(id: Uuid, dst: String) -> Result<(), Error> {

    let mut index_list: Vec<(Uuid, String)> = get_index()?
        .into_iter()
        .filter(|x| x.0 != id)
        .collect();

    index_list.push((id, dst));
    write(&index_list)

}
