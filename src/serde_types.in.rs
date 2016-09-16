#[cfg(test)]
mod test {
    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub enum T {
        A(usize),
        B,
        C(i8, i8),
        D { a: isize },
    }
}
