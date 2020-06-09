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


use petgraph::graph::{Graph, NodeIndex};
use petgraph::Direction::Outgoing;
use ansi_term::Colour::Blue;

use crate::errors::Result;
use crate::pass;

pub fn list() -> Result<()> {
    let mut index_list = pass::index::get_index()?;
    index_list.sort_by(|a, b| b.1.to_lowercase().cmp(&a.1.to_lowercase()));
    let (graph, root) = pass::index::to_graph(&index_list);
    let mut open: Vec<TreeFmtOpen> = Vec::new();
    recursive_tree_print(&graph, root, &mut open);
    Ok(())
}

enum TreeFmtOpen {
    Line,
    Last
}

fn print_with_level(part: &str, open: &Vec<TreeFmtOpen>, is_dir: bool) {
    let mut space = String::new();
    if open.len() >= 2 {
        for o in open[0..open.len()-1].into_iter() {
            match o {
                TreeFmtOpen::Line  => space.push_str("│   "),
                TreeFmtOpen::Last  => space.push_str("    ")
            }
        }
    }
    if open.len() >= 1 {
        match open[open.len() - 1] {
            TreeFmtOpen::Line  => space.push_str("├── "),
            TreeFmtOpen::Last  => space.push_str("└── ")
        }
    }
    if is_dir {
        println!("{}{}", space, Blue.paint(part));
    } else {
        println!("{}{}", space, part);
    }
}

fn recursive_tree_print(graph: &Graph<&str, ()>, node: NodeIndex, open: &mut Vec<TreeFmtOpen>) {
    let mut walker = graph.neighbors_directed(node, Outgoing).detach();
    if let Some(mut child) = walker.next_node(graph) {
        print_with_level(graph.node_weight(node).unwrap(), open, true);
        while let Some(next) = walker.next_node(graph) {
            open.push(TreeFmtOpen::Line);
            recursive_tree_print(graph, child, open);
            child = next;
            open.pop();
        }
        open.push(TreeFmtOpen::Last);
        recursive_tree_print(graph, child, open);
        open.pop();
    } else {
        print_with_level(graph.node_weight(node).unwrap(), open, false);
    }
}

