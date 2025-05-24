use url::Url;

pub fn lmfdb_data_resolve(path: &str) -> Url {
    Url::parse("https://beta.lmfdb.org:443/data/riemann-zeta-zeros/")
        .unwrap()
        .join(path)
        .expect("Cannot join URLs.")
}
