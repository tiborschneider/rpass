use std::panic;
use std::io::{Error, ErrorKind, Write};
use std::process::{Command, Stdio};
use std::collections::HashMap;

use petgraph::graph::{Graph, NodeIndex};

use uuid::Uuid;

use crate::pass::entry::ROOT_FOLDER;

const INDEX_KEY: &str = "uuids/index";

#[allow(dead_code)]
pub fn get_index() -> Result<Vec<(Uuid, String)>, Error> {
    // execute pass command
    let result_utf8 = Command::new("pass")
        .arg(INDEX_KEY)
        .output()?
        .stdout;

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

pub fn get_path_list() -> Result<Vec<String>, Error> {
    let mut path_list: Vec<String> = get_index()?.into_iter().map(|x| x.1).collect();
    path_list.sort();
    Ok(path_list)
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

pub fn insert(id: Uuid, path: &String) -> Result<(), Error> {

    let index_list = get_index()?;

    let mut p = Command::new("pass")
        .arg("insert")
        .arg("--multiline")
        .arg(INDEX_KEY)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;

    if let Some(mut writer) = p.stdin.take() {
        // write the first element
        writer.write_all(&format!("{} {}", id, path).into_bytes())?;
        
        for (id, path) in index_list {
            writer.write_all("\n".as_bytes())?;
            writer.write_all(&format!("{} {}", id, path).into_bytes())?;
        }
    }

    p.wait()?;

    Ok(())

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
        .arg(format!("{}/{}", ROOT_FOLDER, id))
        .output()?;

    // rewrite the index list
    let mut p = Command::new("pass")
        .arg("insert")
        .arg("--multiline")
        .arg(INDEX_KEY)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;

    if let Some(mut writer) = p.stdin.take() {
        // write the first element
        let mut index_iter = index_list.into_iter();
        let (id, path) = index_iter.next().unwrap();
        writer.write_all(&format!("{} {}", id, path).into_bytes())?;

        // write all other elements
        for (id, path) in index_iter {
            writer.write_all("\n".as_bytes())?;
            writer.write_all(&format!("{} {}", id, path).into_bytes())?;
        }
    }

    p.wait()?;

    Ok(())

}

pub fn mv(id: Uuid, dst: String) -> Result<(), Error> {
    let index_list: Vec<(Uuid, String)> = get_index()?;

    let mut index_list_mod: Vec<(Uuid, String)> = Vec::new();

    for (i, s) in index_list {
        if i == id {
            index_list_mod.push((i, dst.clone()));
        } else if s == dst {
            return Err(Error::new(ErrorKind::AlreadyExists, "Destination already exists!"));
        } else {
            index_list_mod.push((i, s));
        }
    }

    // rewrite the index list
    let mut p = Command::new("pass")
        .arg("insert")
        .arg("--multiline")
        .arg(INDEX_KEY)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;

    if let Some(mut writer) = p.stdin.take() {
        // write the first element
        let mut index_iter = index_list_mod.into_iter();
        let (id, path) = index_iter.next().unwrap();
        writer.write_all(&format!("{} {}", id, path).into_bytes())?;

        // write all other elements
        for (id, path) in index_iter {
            writer.write_all("\n".as_bytes())?;
            writer.write_all(&format!("{} {}", id, path).into_bytes())?;
        }
    }

    p.wait()?;

    Ok(())
}
