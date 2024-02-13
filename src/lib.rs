/*
 * Copyright (c) 2024 Matteo Franceschini
 * All rights reserved.
 *
 * Use of this source code is governed by BSD-3-Clause-Clear
 * license that can be found in the LICENSE file
 */

//! # Readingtime
//! Extract data from HTML and calculate the corresponding reading time using the passed word per minute count.

use select::{document::Document, node::Node, predicate::Text};

/// Calculate the reading time of the given html content and the given wpm.
pub fn calculate_reading_time(content: &str, wpm: f32) -> f32 {
    let html = Document::from(content);
    let nodes: Vec<Node> = html
        .find(Text)
        .filter(_is_parent_visible)
        .filter(_is_visible)
        .filter(_is_empty)
        .collect();
    let text: Vec<String> = nodes.iter().map(|node| node.text()).collect();
    let text = text.join(" ");
    let words: Vec<&str> = text.split_whitespace().collect();
    let words_count: f32 = words.len() as f32;
    (words_count / wpm).ceil()
}

fn _is_parent_visible(node: &Node) -> bool {
    let invisible = ["head", "style", "script", "title"];
    match node.parent() {
        Some(parent) => match parent.name() {
            Some(parent_name) => !invisible.contains(&parent_name),
            None => false,
        },
        None => false,
    }
}

fn _is_visible(node: &Node) -> bool {
    let invisible = ["link", "iframe"];
    match node.name() {
        Some(name) => !invisible.contains(&name),
        None => true,
    }
}

fn _is_empty(node: &Node) -> bool {
    match node.as_text() {
        Some(content) => !matches!(content, " " | ""),
        None => false,
    }
}
