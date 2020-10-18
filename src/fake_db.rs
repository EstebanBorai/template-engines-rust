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

    pub fn get_person_by_id(&self, id: u32) -> Option<Person> {
        self.persons.clone().into_iter().find(|p| p.id == id)
    }

    pub fn delete(&mut self, id: u32) -> bool {
        if let Some(person_index) = self.persons.iter().position(|p| p.id == id) {
            self.persons.remove(person_index);

            return true;
        }

        false
    }

    pub fn insert(&mut self, name: &str) -> u32 {
        let next_id = self.persons.len() + 1;
        let next_id = next_id as u32;

        self.persons.push(Person::new(next_id, name));

        next_id
    }

    pub fn update(&mut self, id: u32, name: &str) -> u32 {
        if let Some((index, _)) = self
            .persons
            .iter()
            .enumerate()
            .find(|(_, person)| person.id == id) {
                self.persons[index] = Person::new(id, name);

                return id;
            }

        return 0;
    }
}
