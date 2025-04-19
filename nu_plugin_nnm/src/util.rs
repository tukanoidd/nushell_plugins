#[macro_export]
macro_rules! run_with_nnm {
    (|$zbus:ident, $nm:ident| $block:block) => {{
        use $crate::types::*;

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .to_labeled()?;

        runtime.block_on(async move {
            let $zbus = zbus::Connection::system().await.to_labeled()?;
            let $nm = NMConnection::new(&$zbus).await?;

            $block
        })
    }};
}
