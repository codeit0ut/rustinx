#[derive(Debug)]
pub enum RouteError {
    InvalidPath,
}

#[derive(Debug)]
pub enum RouteTarget {
    Static,
    Proxy,
}

pub fn route_resolver(path: &str) -> Result<RouteTarget, RouteError> {

    if path.starts_with("/api") {
        Ok(RouteTarget::Proxy)
    } else if path.starts_with("/") {
        Ok(RouteTarget::Static)
    } else {
        Err(RouteError::InvalidPath)
    }

}