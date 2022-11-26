use super::types::Service;

// order services by their dependencies
// unsure of the accuracy of this algorithm but its fine for now
pub fn order_by_dependencies(services: &mut [(&String, &Service)]) {
    services.sort_by(|(a_name, a_service), (b_name, b_service)| {
        let a_depends_on = a_service.depends_on.clone();
        let b_depends_on = b_service.depends_on.clone();

        if a_depends_on.is_none() && b_depends_on.is_none() {
            return std::cmp::Ordering::Equal;
        }

        if a_depends_on.is_none() {
            return std::cmp::Ordering::Less;
        }

        if b_depends_on.is_none() {
            return std::cmp::Ordering::Greater;
        }

        let a_depends_on = a_depends_on.unwrap();
        let b_depends_on = b_depends_on.unwrap();

        if a_depends_on.contains(b_name) {
            return std::cmp::Ordering::Less;
        }

        if b_depends_on.contains(a_name) {
            return std::cmp::Ordering::Greater;
        }

        std::cmp::Ordering::Equal
    });
}
