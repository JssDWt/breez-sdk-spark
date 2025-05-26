use anyhow::Result;
use hickory_resolver::TokioResolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::name_server::TokioConnectionProvider;
use lazy_static::lazy_static;

lazy_static! {
    static ref DNS_RESOLVER: TokioResolver = {
        let mut opts = ResolverOpts::default();
        opts.validate = true;

        TokioResolver::builder_with_config(
            ResolverConfig::default(),
            TokioConnectionProvider::default(),
        )
        .with_options(opts)
        .build()
    };
}

pub(crate) async fn txt_lookup(dns_name: String) -> Result<Vec<String>> {
    let txt_lookup = DNS_RESOLVER.txt_lookup(dns_name).await?;
    let records: Vec<String> = txt_lookup.iter().map(|r| r.to_string()).collect();
    Ok(records)
}
