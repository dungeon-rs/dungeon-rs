//! Small helpers for the words the Managers and the Editor give the Author.

/// `number` written with a comma between each group of three digits: `100000` as `100,000`.
#[must_use]
pub fn grouped(number: u64) -> String {
    let digits = number.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    grouped
}
