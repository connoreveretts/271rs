// Even Fibonacci numbers
fn main() {
    // Calculate the sum of even Fibonacci numbers not exceeding four million
    let mut sum = 0;
    let mut prev = 0;
    let mut curr = 1;
    // Generate Fibonacci numbers and sum the even ones
    while curr < 4000000 {
        // Check if the current Fibonacci number is even
        if curr % 2 == 0 {
            sum += curr;
        }
        // Update to the next Fibonacci number
        let tmp = curr + prev;
        prev = curr;
        curr = tmp;
    }
    // Print the result
    println!("{}", sum);
}