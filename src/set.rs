#[derive(Debug, Clone)]
pub struct Set<T> {
    data: Vec<T>,
}

impl<T: PartialEq> PartialEq for Set<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data.len() == other.data.len() && self.data.iter().all(|x| other.data.contains(x))
    }
}

impl<T: PartialEq> Set<T> {
    pub fn new() -> Self {
        Set { data: Vec::new() }
    }

    pub fn add(&mut self, item: T) {
        if !self.data.contains(&item) {
            self.data.push(item);
        }
    }

    pub fn remove(&mut self, item: T) {
        if let Some(index) = self.data.iter().position(|x| x == &item) {
            self.data.remove(index);
        }
    }

    pub fn contains(&self, item: &T) -> bool {
        self.data.contains(item)
    }

    pub fn iter(&self) -> std::slice::Iter<T> {
        self.data.iter()
    }

    pub fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for item in iter {
            self.add(item);
        }
    }

    pub fn intersection(&self, other: &Self) -> Self
    where
        T: Clone,
    {
        let data = self
            .data
            .iter()
            .filter(|x| other.contains(x))
            .cloned()
            .collect();
        Set { data }
    }

    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }
}

impl<T> IntoIterator for Set<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<T: PartialEq> FromIterator<T> for Set<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = Set::new();
        for item in iter {
            set.add(item);
        }
        set
    }
}

impl<T> std::ops::Deref for Set<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
