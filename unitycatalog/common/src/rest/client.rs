use cloud_client::CloudClient;
use futures::stream::BoxStream;
use futures::{Future, Stream, StreamExt, TryStreamExt};
use reqwest::IntoUrl;

pub use crate::api::catalogs::CatalogClient;
pub use crate::api::credentials::CredentialsClient;
pub use crate::api::external_locations::ExternalLocationsClient;
pub use crate::api::recipients::RecipientsClient;
pub use crate::api::schemas::SchemasClient;
pub use crate::api::shares::SharesClient;
pub use crate::api::tables::TablesClient;
use crate::models::catalogs::v1 as catalog;
use crate::models::credentials::v1 as cred;
use crate::models::credentials::v1::Purpose;
use crate::models::external_locations::v1 as loc;
use crate::models::recipients::v1 as rec;
use crate::models::schemas::v1 as schema;
use crate::models::shares::v1 as share;
use crate::models::tables::v1 as tbl;
use crate::{Error, Result};

#[derive(Clone)]
pub struct UnityCatalogClient {
    client: CloudClient,
    base_url: url::Url,
}

impl UnityCatalogClient {
    pub fn new(client: CloudClient, base_url: url::Url) -> Self {
        Self { client, base_url }
    }

    pub fn catalogs(&self) -> CatalogClient {
        CatalogClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn credentials(&self) -> CredentialsClient {
        CredentialsClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn external_locations(&self) -> ExternalLocationsClient {
        ExternalLocationsClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn recipients(&self) -> RecipientsClient {
        RecipientsClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn schemas(&self) -> SchemasClient {
        SchemasClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn tables(&self) -> TablesClient {
        TablesClient::new(self.client.clone(), self.base_url.clone())
    }

    pub fn shares(&self) -> SharesClient {
        SharesClient::new(self.client.clone(), self.base_url.clone())
    }
}

impl CatalogClient {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<catalog::CatalogInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = catalog::ListCatalogsRequest {
                max_results,
                page_token,
            };
            let res = self
                .list_catalogs(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.catalogs, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        comment: impl Into<Option<String>>,
    ) -> Result<catalog::CatalogInfo> {
        let request = catalog::CreateCatalogRequest {
            name: name.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_catalog(&request).await
    }

    pub async fn get(&self, name: impl Into<String>) -> Result<catalog::CatalogInfo> {
        let request = catalog::GetCatalogRequest {
            name: name.into(),
            include_browse: None,
        };
        self.get_catalog(&request).await
    }

    pub async fn delete(
        &self,
        name: impl Into<String>,
        force: impl Into<Option<bool>>,
    ) -> Result<()> {
        let request = catalog::DeleteCatalogRequest {
            name: name.into(),
            force: force.into(),
        };
        self.delete_catalog(&request).await
    }
}

impl SchemasClient {
    pub fn list(
        &self,
        catalog_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<schema::SchemaInfo>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        stream_paginated(
            (catalog_name, max_results),
            move |(catalog_name, max_results), page_token| async move {
                let request = schema::ListSchemasRequest {
                    catalog_name: catalog_name.clone(),
                    max_results,
                    page_token,
                    include_browse: None,
                };
                let res = self
                    .list_schemas(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((
                    res.schemas,
                    (catalog_name, max_results),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        catalog_name: impl Into<String>,
        name: impl Into<String>,
        comment: impl Into<Option<String>>,
    ) -> Result<schema::SchemaInfo> {
        let request = schema::CreateSchemaRequest {
            catalog_name: catalog_name.into(),
            name: name.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_schema(&request).await
    }

    pub async fn get(
        &self,
        catalog_name: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<schema::SchemaInfo> {
        let request = schema::GetSchemaRequest {
            full_name: format!("{}.{}", catalog_name.into(), name.into()),
        };
        self.get_schema(&request).await
    }

    pub async fn delete(
        &self,
        catalog_name: impl Into<String>,
        name: impl Into<String>,
        force: impl Into<Option<bool>>,
    ) -> Result<()> {
        let request = schema::DeleteSchemaRequest {
            full_name: format!("{}.{}", catalog_name.into(), name.into()),
            force: force.into(),
        };
        tracing::info!("deleting schema {}", request.full_name);
        self.delete_schema(&request).await
    }
}

impl TablesClient {
    pub fn list_summaries(
        &self,
        catalog_name: impl Into<String>,
        schema_name_pattern: impl Into<Option<String>>,
        table_name_pattern: impl Into<Option<String>>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<tbl::TableSummary>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let schema_name_pattern = schema_name_pattern.into();
        let table_name_pattern = table_name_pattern.into();
        stream_paginated(
            (
                catalog_name,
                schema_name_pattern,
                table_name_pattern,
                max_results,
            ),
            move |(catalog_name, schema_name_pattern, table_name_pattern, max_results),
                  page_token| async move {
                let request = tbl::ListTableSummariesRequest {
                    catalog_name: catalog_name.clone(),
                    schema_name_pattern: schema_name_pattern.clone(),
                    table_name_pattern: table_name_pattern.clone(),
                    page_token,
                    max_results: None,
                    include_manifest_capabilities: None,
                };
                let res = self
                    .list_table_summaries(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((
                    res.tables,
                    (
                        catalog_name,
                        schema_name_pattern,
                        table_name_pattern,
                        max_results,
                    ),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub fn list(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        max_results: impl Into<Option<i32>>,
        include_delta_metadata: impl Into<Option<bool>>,
        omit_columns: impl Into<Option<bool>>,
        omit_properties: impl Into<Option<bool>>,
        omit_username: impl Into<Option<bool>>,
    ) -> BoxStream<'_, Result<tbl::TableInfo>> {
        let max_results = max_results.into();
        let catalog_name = catalog_name.into();
        let schema_name = schema_name.into();
        let include_delta_metadata = include_delta_metadata.into();
        let omit_columns = omit_columns.into();
        let omit_properties = omit_properties.into();
        let omit_username = omit_username.into();
        stream_paginated(
            (catalog_name, schema_name, max_results),
            move |(catalog_name, schema_name, max_results), page_token| async move {
                let request = tbl::ListTablesRequest {
                    catalog_name: catalog_name.clone(),
                    schema_name: schema_name.clone(),
                    include_delta_metadata,
                    omit_columns,
                    omit_properties,
                    omit_username,
                    max_results,
                    page_token,
                    include_browse: None,
                    include_manifest_capabilities: None,
                };
                let res = self
                    .list_tables(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;
                Ok((
                    res.tables,
                    (catalog_name, schema_name, max_results),
                    res.next_page_token,
                ))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        name: impl Into<String>,
        comment: impl Into<Option<String>>,
    ) -> Result<tbl::TableInfo> {
        let request = tbl::CreateTableRequest {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            name: name.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_table(&request).await
    }

    pub async fn get(
        &self,
        full_name: impl Into<String>,
        include_delta_metadata: impl Into<Option<bool>>,
    ) -> Result<tbl::TableInfo> {
        let request = tbl::GetTableRequest {
            full_name: full_name.into(),
            include_delta_metadata: include_delta_metadata.into(),
            include_browse: None,
            include_manifest_capabilities: None,
        };
        self.get_table(&request).await
    }

    pub async fn delete(&self, full_name: impl Into<String>) -> Result<()> {
        let request = tbl::DeleteTableRequest {
            full_name: full_name.into(),
        };
        self.delete_table(&request).await
    }
}

impl CredentialsClient {
    pub fn list(
        &self,
        purpose: Option<Purpose>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<cred::CredentialInfo>> {
        let max_results = max_results.into();
        let purpose = purpose.map(|p| p as i32);
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = cred::ListCredentialsRequest {
                max_results,
                page_token,
                purpose,
            };
            let res = self
                .list_credentials(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.credentials, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        purpose: Purpose,
        comment: impl Into<Option<String>>,
    ) -> Result<cred::CredentialInfo> {
        let request = cred::CreateCredentialRequest {
            name: name.into(),
            purpose: purpose.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_credential(&request).await
    }

    pub async fn get(&self, name: impl Into<String>) -> Result<cred::CredentialInfo> {
        let request = cred::GetCredentialRequest { name: name.into() };
        self.get_credential(&request).await
    }

    pub async fn delete(&self, name: impl Into<String>) -> Result<()> {
        let request = cred::DeleteCredentialRequest { name: name.into() };
        self.delete_credential(&request).await
    }
}

impl ExternalLocationsClient {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<loc::ExternalLocationInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = loc::ListExternalLocationsRequest {
                max_results,
                page_token,
                include_browse: None,
            };
            let res = self
                .list_external_locations(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.external_locations, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        url: impl IntoUrl,
        credential_name: impl Into<String>,
        comment: impl Into<Option<String>>,
    ) -> Result<loc::ExternalLocationInfo> {
        let request = loc::CreateExternalLocationRequest {
            name: name.into(),
            url: url
                .into_url()
                .map(|u| u.to_string())
                .map_err(|e| Error::generic(e.to_string()))?,
            credential_name: credential_name.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_external_location(&request).await
    }

    pub async fn get(&self, name: impl Into<String>) -> Result<loc::ExternalLocationInfo> {
        let request = loc::GetExternalLocationRequest { name: name.into() };
        self.get_external_location(&request).await
    }

    pub async fn delete(
        &self,
        name: impl Into<String>,
        force: impl Into<Option<bool>>,
    ) -> Result<()> {
        let request = loc::DeleteExternalLocationRequest {
            name: name.into(),
            force: force.into(),
        };
        self.delete_external_location(&request).await
    }
}

impl RecipientsClient {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<rec::RecipientInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = rec::ListRecipientsRequest {
                max_results,
                page_token,
            };
            let res = self
                .list_recipients(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.recipients, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        authentication_type: rec::AuthenticationType,
        comment: impl Into<Option<String>>,
    ) -> Result<rec::RecipientInfo> {
        let request = rec::CreateRecipientRequest {
            name: name.into(),
            authentication_type: authentication_type.into(),
            comment: comment.into(),
            ..Default::default()
        };
        self.create_recipient(&request).await
    }

    pub async fn get(&self, name: impl Into<String>) -> Result<rec::RecipientInfo> {
        let request = rec::GetRecipientRequest { name: name.into() };
        self.get_recipient(&request).await
    }

    pub async fn delete(&self, name: impl Into<String>) -> Result<()> {
        let request = rec::DeleteRecipientRequest { name: name.into() };
        self.delete_recipient(&request).await
    }
}

impl SharesClient {
    pub fn list(
        &self,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<share::ShareInfo>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |max_results, page_token| async move {
            let request = share::ListSharesRequest {
                max_results,
                page_token,
            };
            let res = self
                .list_shares(&request)
                .await
                .map_err(|e| Error::generic(e.to_string()))?;
            Ok((res.shares, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn create(
        &self,
        name: impl Into<String>,
        comment: impl Into<Option<String>>,
    ) -> Result<share::ShareInfo> {
        let request = share::CreateShareRequest {
            name: name.into(),
            comment: comment.into(),
        };
        self.create_share(&request).await
    }

    pub async fn get(
        &self,
        name: impl Into<String>,
        include_shared_data: impl Into<Option<bool>>,
    ) -> Result<share::ShareInfo> {
        let request = share::GetShareRequest {
            name: name.into(),
            include_shared_data: include_shared_data.into(),
        };
        self.get_share(&request).await
    }

    pub async fn delete(&self, name: impl Into<String>) -> Result<()> {
        let request = share::DeleteShareRequest { name: name.into() };
        self.delete_share(&request).await
    }

    pub async fn update(
        &self,
        name: impl Into<String>,
        new_name: impl Into<Option<String>>,
        updates: Vec<share::DataObjectUpdate>,
        comment: impl Into<Option<String>>,
        owner: impl Into<Option<String>>,
    ) -> Result<share::ShareInfo> {
        let request = share::UpdateShareRequest {
            name: name.into(),
            new_name: new_name.into().and_then(|s| (!s.is_empty()).then_some(s)),
            comment: comment.into(),
            owner: owner.into(),
            updates,
        };
        self.update_share(&request).await
    }
}

pub fn stream_paginated<F, Fut, S, T>(state: S, op: F) -> impl Stream<Item = Result<T>>
where
    F: Fn(S, Option<String>) -> Fut + Copy,
    Fut: Future<Output = Result<(T, S, Option<String>)>>,
{
    enum PaginationState<T> {
        Start(T),
        HasMore(T, String),
        Done,
    }

    futures::stream::unfold(PaginationState::Start(state), move |state| async move {
        let (s, page_token) = match state {
            PaginationState::Start(s) => (s, None),
            PaginationState::HasMore(s, page_token) if !page_token.is_empty() => {
                (s, Some(page_token))
            }
            _ => {
                return None;
            }
        };

        let (resp, s, continuation) = match op(s, page_token).await {
            Ok(resp) => resp,
            Err(e) => return Some((Err(e), PaginationState::Done)),
        };

        let next_state = match continuation {
            Some(token) => PaginationState::HasMore(s, token),
            None => PaginationState::Done,
        };

        Some((Ok(resp), next_state))
    })
}
