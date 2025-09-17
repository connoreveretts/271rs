use std::fs::File;
use std::io::{self, Read, Write};

// ANSI color codes
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

// Box drawing characters
const TOP: &str = "┌───┬───┬───┬───┬───┐";
const MIDDLE: &str = "├───┼───┼───┼───┼───┤";
const BOTTOM: &str = "└───┴───┴───┴───┴───┘";

// Word list
const WORDS: &[&str] = &[
    "sator", "arepo", "tenet", "opera", "rotas", "about", "other", "which",
    "their", "would", "there", "could", "first", "after", "these", "being",
    "where", "every", "right", "think", "three", "never", "come", "made",
    "also", "back", "good", "woman", "through", "just", "form", "great",
    "say", "help", "low", "line", "turn", "cause", "much", "mean",
    "before", "move", "boy", "old", "too", "same", "tell", "does",
    "set", "want", "air", "well", "play", "small", "end", "home"
];

fn main() {
    println!("Use lowercase only btw.");
    
    // Get random word
    let target_word = get_random_word();
    
    // Store all guesses
    let mut guesses: Vec<String> = Vec::new();
    let max_attempts = 6;
    let mut won = false;
    
    // Game loop
    for _attempt in 0..max_attempts {
        print!("Enter your guess: ");
        io::stdout().flush().unwrap();
        
        let mut guess = String::new();
        io::stdin().read_line(&mut guess).unwrap();
        let guess = guess.trim().to_string();
        
        // Validate guess length
        if guess.len() != 5 {
            println!("Word must be 5 letters!");
            continue;
        }
        
        // Validate guess is in word list
        if !WORDS.contains(&guess.as_str()) {
            println!("Not a valid word!");
            continue;
        }
        
        // Add guess to list
        guesses.push(guess.clone());
        
        // Draw the board
        draw_board(&guesses, &target_word);
        
        // Check if won
        if guess == target_word {
            won = true;
            break;
        }
    }
    
    // Print final message
    if won {
        println!("Winner");
    } else {
        println!("Game over :(");
        println!("The word was: {}", target_word);
    }
}

fn get_random_word() -> String {
    // Try to read from /dev/random
    let random_byte = match File::open("/dev/random") {
        Ok(mut file) => {
            let mut buffer = [0u8; 1];
            match file.read_exact(&mut buffer) {
                Ok(_) => buffer[0],
                Err(_) => {
                    // Fallback to /dev/urandom
                    match File::open("/dev/urandom") {
                        Ok(mut urandom_file) => {
                            let mut urandom_buffer = [0u8; 1];
                            match urandom_file.read_exact(&mut urandom_buffer) {
                                Ok(_) => urandom_buffer[0],
                                Err(_) => 42, // Hardcoded fallback
                            }
                        }
                        Err(_) => 42, // Hardcoded fallback
                    }
                }
            }
        }
        Err(_) => {
            // Fallback to /dev/urandom
            match File::open("/dev/urandom") {
                Ok(mut urandom_file) => {
                    let mut urandom_buffer = [0u8; 1];
                    match urandom_file.read_exact(&mut urandom_buffer) {
                        Ok(_) => urandom_buffer[0],
                        Err(_) => 42, // Hardcoded fallback
                    }
                }
                Err(_) => 42, // Hardcoded fallback
            }
        }
    };
    
    // Use the random byte to select a word
    let index = (random_byte as usize) % WORDS.len();
    WORDS[index].to_string()
}

fn draw_board(guesses: &Vec<String>, target_word: &str) {
    println!("{}", TOP);
    
    for (i, guess) in guesses.iter().enumerate() {
        print_word_with_colors(guess, target_word);
        
        // Print the appropriate divider
        if i < guesses.len() - 1 {
            println!("{}", MIDDLE);
        }
    }
    
    println!("{}", BOTTOM);
}

fn print_word_with_colors(guess: &str, target: &str) {
    print!("│");
    
    let guess_chars: Vec<char> = guess.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();
    
    for i in 0..5 {
        let color = get_color_for_position(&guess_chars, &target_chars, i);
        print!(" {}{}{} │", color, guess_chars[i].to_uppercase(), RESET);
    }
    println!();
}

fn get_color_for_position(guess_chars: &Vec<char>, target_chars: &Vec<char>, position: usize) -> &'static str {
    let current_char = guess_chars[position];
    
    // Check if letter is in correct position (GREEN)
    if current_char == target_chars[position] {
        return GREEN;
    }
    
    // Check if letter exists in the word but wrong position (YELLOW)
    // We need to handle duplicates correctly
    let mut target_counts = [0; 26];
    let mut guess_green_counts = [0; 26];
    
    // Count letters in target
    for ch in target_chars {
        if ch.is_ascii_lowercase() {
            target_counts[(*ch as usize) - ('a' as usize)] += 1;
        }
    }
    
    // First pass: mark green positions and subtract from counts
    for i in 0..5 {
        if guess_chars[i] == target_chars[i] && guess_chars[i].is_ascii_lowercase() {
            guess_green_counts[(guess_chars[i] as usize) - ('a' as usize)] += 1;
        }
    }
    
    // Adjust available counts for yellows
    for i in 0..26 {
        target_counts[i] -= guess_green_counts[i];
    }
    
    // Second pass: check if current position should be yellow
    if current_char.is_ascii_lowercase() {
        let char_index = (current_char as usize) - ('a' as usize);
        
        // Count yellows before this position
        let mut yellows_before = 0;
        for i in 0..position {
            if guess_chars[i] == current_char && guess_chars[i] != target_chars[i] {
                yellows_before += 1;
            }
        }
        
        // Check if we can still mark this as yellow
        if yellows_before < target_counts[char_index] {
            return YELLOW;
        }
    }
    
    // Letter not in word (RED)
    RED
}