use futures::TryStreamExt;
use prost::Message;
use tauri::State;
use unitycatalog_common::models::catalogs::v1::{
    CatalogInfo, CreateCatalogRequest, UpdateCatalogRequest,
};
use unitycatalog_common::models::credentials::v1::{
    CreateCredentialRequest, CredentialInfo, Purpose, UpdateCredentialRequest,
};
use unitycatalog_common::models::external_locations::v1::{
    CreateExternalLocationRequest, ExternalLocationInfo, UpdateExternalLocationRequest,
};
use unitycatalog_common::models::recipients::v1::{
    CreateRecipientRequest, RecipientInfo, UpdateRecipientRequest,
};
use unitycatalog_common::models::schemas::v1::{
    CreateSchemaRequest, SchemaInfo, UpdateSchemaRequest,
};
use unitycatalog_common::models::shares::v1::{CreateShareRequest, ShareInfo, UpdateShareRequest};
use unitycatalog_common::models::tables::v1::{CreateTableRequest, TableInfo, TableSummary};
use unitycatalog_common::rest::client::UnityCatalogClient;

use crate::error::Result;

#[tauri::command]
pub async fn list_catalogs(
    state: State<'_, UnityCatalogClient>,
    max_results: Option<i32>,
) -> Result<Vec<CatalogInfo>> {
    Ok(state.catalogs().list(max_results).try_collect().await?)
}

#[tauri::command]
pub async fn get_catalog(
    state: State<'_, UnityCatalogClient>,
    name: String,
) -> Result<CatalogInfo> {
    Ok(state.catalogs().get(name).await?)
}

#[tauri::command]
pub async fn create_catalog(
    state: State<'_, UnityCatalogClient>,
    request: CreateCatalogRequest,
) -> Result<CatalogInfo> {
    Ok(state.catalogs().create_catalog(&request).await?)
}

#[tauri::command]
pub async fn update_catalog(
    state: State<'_, UnityCatalogClient>,
    request: Vec<u8>,
) -> Result<CatalogInfo> {
    let request = UpdateCatalogRequest::decode(request.as_slice())?;
    Ok(state.catalogs().update_catalog(&request).await?)
}

#[tauri::command]
pub async fn delete_catalog(
    state: State<'_, UnityCatalogClient>,
    name: String,
    force: Option<bool>,
) -> Result<()> {
    Ok(state.catalogs().delete(name, force).await?)
}

#[tauri::command]
pub async fn list_schemas(
    state: State<'_, UnityCatalogClient>,
    catalog: String,
    max_results: Option<i32>,
) -> Result<Vec<SchemaInfo>> {
    Ok(state
        .schemas()
        .list(catalog, max_results)
        .try_collect()
        .await?)
}

#[tauri::command]
pub async fn get_schema(
    state: State<'_, UnityCatalogClient>,
    catalog: String,
    name: String,
) -> Result<SchemaInfo> {
    Ok(state.schemas().get(catalog, name).await?)
}

#[tauri::command]
pub async fn create_schema(
    state: State<'_, UnityCatalogClient>,
    request: CreateSchemaRequest,
) -> Result<SchemaInfo> {
    Ok(state.schemas().create_schema(&request).await?)
}

#[tauri::command]
pub async fn update_schema(
    state: State<'_, UnityCatalogClient>,
    request: Vec<u8>,
) -> Result<SchemaInfo> {
    let request = UpdateSchemaRequest::decode(request.as_slice())?;
    Ok(state.schemas().update_schema(&request).await?)
}

#[tauri::command]
pub async fn delete_schema(
    state: State<'_, UnityCatalogClient>,
    catalog: String,
    name: String,
    force: Option<bool>,
) -> Result<()> {
    Ok(state.schemas().delete(catalog, name, force).await?)
}

#[tauri::command]
pub async fn list_credentials(
    state: State<'_, UnityCatalogClient>,
    purpose: Option<Purpose>,
    max_results: Option<i32>,
) -> Result<Vec<CredentialInfo>> {
    Ok(state
        .credentials()
        .list(purpose, max_results)
        .try_collect()
        .await?)
}

#[tauri::command]
pub async fn get_credential(
    state: State<'_, UnityCatalogClient>,
    name: String,
) -> Result<CredentialInfo> {
    Ok(state.credentials().get(name).await?)
}

#[tauri::command]
pub async fn create_credential(
    state: State<'_, UnityCatalogClient>,
    request: CreateCredentialRequest,
) -> Result<CredentialInfo> {
    Ok(state.credentials().create_credential(&request).await?)
}

#[tauri::command]
pub async fn update_credential(
    state: State<'_, UnityCatalogClient>,
    request: Vec<u8>,
) -> Result<CredentialInfo> {
    let request = UpdateCredentialRequest::decode(request.as_slice())?;
    Ok(state.credentials().update_credential(&request).await?)
}

#[tauri::command]
pub async fn delete_credential(state: State<'_, UnityCatalogClient>, name: String) -> Result<()> {
    Ok(state.credentials().delete(name).await?)
}

#[tauri::command]
pub async fn list_external_locations(
    state: State<'_, UnityCatalogClient>,
    max_results: Option<i32>,
) -> Result<Vec<ExternalLocationInfo>> {
    Ok(state
        .external_locations()
        .list(max_results)
        .try_collect()
        .await?)
}

#[tauri::command]
pub async fn get_external_location(
    state: State<'_, UnityCatalogClient>,
    name: String,
) -> Result<ExternalLocationInfo> {
    Ok(state.external_locations().get(name).await?)
}

#[tauri::command]
pub async fn create_external_location(
    state: State<'_, UnityCatalogClient>,
    request: CreateExternalLocationRequest,
) -> Result<ExternalLocationInfo> {
    Ok(state
        .external_locations()
        .create_external_location(&request)
        .await?)
}

#[tauri::command]
pub async fn update_external_location(
    state: State<'_, UnityCatalogClient>,
    request: Vec<u8>,
) -> Result<ExternalLocationInfo> {
    let request = UpdateExternalLocationRequest::decode(request.as_slice())?;
    Ok(state
        .external_locations()
        .update_external_location(&request)
        .await?)
}

#[tauri::command]
pub async fn delete_external_location(
    state: State<'_, UnityCatalogClient>,
    name: String,
    force: Option<bool>,
) -> Result<()> {
    Ok(state.external_locations().delete(name, force).await?)
}

#[tauri::command]
pub async fn list_recipients(
    state: State<'_, UnityCatalogClient>,
    max_results: Option<i32>,
) -> Result<Vec<RecipientInfo>> {
    Ok(state.recipients().list(max_results).try_collect().await?)
}

#[tauri::command]
pub async fn get_recipient(
    state: State<'_, UnityCatalogClient>,
    name: String,
) -> Result<RecipientInfo> {
    Ok(state.recipients().get(name).await?)
}

#[tauri::command]
pub async fn create_recipient(
    state: State<'_, UnityCatalogClient>,
    request: CreateRecipientRequest,
) -> Result<RecipientInfo> {
    Ok(state.recipients().create_recipient(&request).await?)
}

#[tauri::command]
pub async fn update_recipient(
    state: State<'_, UnityCatalogClient>,
    request: Vec<u8>,
) -> Result<RecipientInfo> {
    let request = UpdateRecipientRequest::decode(request.as_slice())?;
    Ok(state.recipients().update_recipient(&request).await?)
}

#[tauri::command]
pub async fn delete_recipient(state: State<'_, UnityCatalogClient>, name: String) -> Result<()> {
    Ok(state.recipients().delete(name).await?)
}

#[tauri::command]
pub async fn list_shares(
    state: State<'_, UnityCatalogClient>,
    max_results: Option<i32>,
) -> Result<Vec<ShareInfo>> {
    Ok(state.shares().list(max_results).try_collect().await?)
}

#[tauri::command]
pub async fn get_share(
    state: State<'_, UnityCatalogClient>,
    name: String,
    include_shared_data: Option<bool>,
) -> Result<ShareInfo> {
    Ok(state.shares().get(name, include_shared_data).await?)
}

#[tauri::command]
pub async fn create_share(
    state: State<'_, UnityCatalogClient>,
    request: CreateShareRequest,
) -> Result<ShareInfo> {
    Ok(state.shares().create_share(&request).await?)
}

#[tauri::command]
pub async fn update_share(
    state: State<'_, UnityCatalogClient>,
    request: Vec<u8>,
) -> Result<ShareInfo> {
    let request = UpdateShareRequest::decode(request.as_slice())?;
    Ok(state.shares().update_share(&request).await?)
}

#[tauri::command]
pub async fn delete_share(state: State<'_, UnityCatalogClient>, name: String) -> Result<()> {
    Ok(state.shares().delete(name).await?)
}

#[tauri::command]
pub async fn list_table_summaries(
    state: State<'_, UnityCatalogClient>,
    catalog: String,
    schema_pattern: Option<String>,
    table_pattern: Option<String>,
    max_results: Option<i32>,
) -> Result<Vec<TableSummary>> {
    Ok(state
        .tables()
        .list_summaries(catalog, schema_pattern, table_pattern, max_results)
        .try_collect()
        .await?)
}

#[tauri::command]
pub async fn list_tables(
    state: State<'_, UnityCatalogClient>,
    catalog: String,
    schema: String,
    max_results: Option<i32>,
    omit_columns: Option<bool>,
    omit_properties: Option<bool>,
    omit_username: Option<bool>,
) -> Result<Vec<TableInfo>> {
    Ok(state
        .tables()
        .list(
            catalog,
            schema,
            max_results,
            None,
            omit_columns,
            omit_properties,
            omit_username,
        )
        .try_collect()
        .await?)
}

#[tauri::command]
pub async fn get_table(
    state: State<'_, UnityCatalogClient>,
    full_name: String,
) -> Result<TableInfo> {
    Ok(state.tables().get(full_name, None).await?)
}

#[tauri::command]
pub async fn create_table(
    state: State<'_, UnityCatalogClient>,
    request: CreateTableRequest,
) -> Result<TableInfo> {
    Ok(state.tables().create_table(&request).await?)
}

#[tauri::command]
pub async fn delete_table(state: State<'_, UnityCatalogClient>, full_name: String) -> Result<()> {
    Ok(state.tables().delete(full_name).await?)
}
