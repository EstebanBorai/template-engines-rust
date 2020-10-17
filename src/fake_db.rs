use crate::person::Person;

pub struct FakeDb {
    persons: Vec<Person>,
}

impl FakeDb {
    pub fn new() -> Self {
        Self {
            persons: vec![
                Person::new(1, "Esteban"),
                Person::new(2, "June"),
                Person::new(3, "Carlos"),
                Person::new(4, "Ana"),
            ],
        }
    }

    pub fn get_all_persons(&self) -> Vec<Person> {
        self.persons.clone()
    }

    pub fn get_persons_by_name<'a>(
        &'a self,
        partial: &'a str,
    ) -> impl Iterator<Item = &Person> + 'a {
        self.persons
            .iter()
            .filter(move |p| p.name.contains(partial))
    }
}
