
#[derive(Clone, Debug)]
pub struct Result {
    pub id: usize,
    pub users: Vec<usize>,
    pub sk: String,
    pub pk: String,
}

impl std::fmt::Display for Result {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut users = String::new();
        for i in &self.users {
            users.push_str(&i.to_string());
            users.push_str(" ");
        }
        write!(f, "id: {}, users: {}, sk: {}, pk: {}",
               self.id, users, self.sk, self.pk)
    }
}