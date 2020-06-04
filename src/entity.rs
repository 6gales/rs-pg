use rs_pg_scheme::Scheme;

pub trait Entity {
	fn scheme() -> Scheme;
}