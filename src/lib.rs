/*
 * Copyright (c) 2024 Matteo Franceschini
 * All rights reserved.
 *
 * Use of this source code is governed by BSD-3-Clause-Clear
 * license that can be found in the LICENSE file
 */

//! # Readingtime
//! Extract data from HTML and calculate the corresponding reading time using the passed word per minute count.

use thiserror::Error;

/// Calculate the reading time of the given html content and the given wpm.
pub fn calculate_reading_time(content: &str, wpm: f32) -> Result<f32, ReadingTimeError> {
    let text = dom_smoothie::Readability::new(content, None, None)
        .map_err(|_| ReadingTimeError::DomSmoothieError)?
        .parse()
        .map_err(|_| ReadingTimeError::DomSmoothieError)?
        .text_content
        .to_string();
    let words: Vec<&str> = text.split_whitespace().collect();
    let words_count: f32 = words.len() as f32;
    let reading_time = (words_count / wpm).ceil();
    Ok(reading_time)
}
#[derive(Debug, Error)]
pub enum ReadingTimeError {
    #[error("The parser failed to grab the article")]
    DomSmoothieError,
}
