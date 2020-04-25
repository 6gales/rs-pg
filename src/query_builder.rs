struct QueryBuilder<T> {
	query: String,
	select_statement: String,
	from_statement: String,
	where_statement: String,
}

impl<T> QueryBuilder<T> where T : Entity {

	fn new() -> Self {
		QueryBuilder{
			query: "SELECT * FROM " + T::name,
		}
	}

	fn select(&mut self) -> Vec<T> {
		vec!()
	}

	fn r#where(&mut self, statement: &String) -> &mut Self {
		self.where_statement += statement;
		self
	}

	fn relation(&mut self) -> &mut Self {

	}
}