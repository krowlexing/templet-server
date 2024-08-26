pub trait TableSql<Insert, Select> {
    fn create() -> &'static str;
    fn select_all() -> &'static str;
    fn delete_by_id() -> &'static str;
    fn insert() -> &'static str;
}
