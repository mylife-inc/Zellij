//! Keep quoted examples intact when a longer fence contains shorter fences.

#[derive(Default)]
pub(crate) struct Fence(Option<(char, usize)>);

impl Fence {
    /// Includes opening/closing lines. A closing fence must match the opening
    /// marker, be at least as long and have no trailing non-whitespace text.
    pub(crate) fn contains(&mut self, line: &str) -> bool {
        let trimmed = line.trim_start();
        let marker = trimmed.chars().next().unwrap_or(' ');
        let count = trimmed.chars().take_while(|c| *c == marker).count();
        if let Some((open, width)) = self.0 {
            if marker == open && count >= width && trimmed[count..].trim().is_empty() {
                self.0 = None;
            }
            return true;
        }
        if matches!(marker, '`' | '~') && count >= 3 {
            self.0 = Some((marker, count));
            return true;
        }
        false
    }
}
