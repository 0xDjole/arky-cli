use crate::client::ArkyClient;
use crate::error::{CliError, Result};
use crate::output::Format;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum MediaCommand {
    /// Upload one or more files (max 50MB total)
    #[command(long_about = "Upload files to the media library.\n\n\
        Accepts one or more file paths. Files are uploaded as multipart form data.\n\
        Returns an array of created media objects with IDs and URLs.\n\n\
        Supported: images (png, jpg, gif, webp, svg), video, PDF, any file type.\n\
        Max total request size: 50MB.\n\n\
        Examples:\n\
        arky media upload photo.jpg\n\
        arky media upload hero.png logo.svg banner.webp\n\
        arky media upload /path/to/document.pdf\n\n\
        Response shape:\n\
        [{\"id\": \"media_abc\", \"mimeType\": \"image/png\", \"title\": \"photo.png\",\n\
          \"resolutions\": {\"original\": {\"url\": \"https://...\"}},\n\
          \"businessId\": \"biz_123\", \"uploadedAt\": \"2025-01-01T00:00:00Z\"}]\n\n\
        Use the returned media ID in relationship_media blocks:\n\
        {\"key\": \"image\", \"type\": \"relationship_media\", \"value\": {\"id\": \"media_abc\"}}")]
    Upload {
        /// File paths to upload
        #[arg(required = true)]
        files: Vec<String>,
    },
    /// List media files
    #[command(long_about = "List media files in the business library.\n\n\
        Returns paginated results with cursor-based pagination.\n\n\
        Examples:\n\
        arky media list\n\
        arky media list --limit 5\n\
        arky media list --mime-type image/png\n\
        arky media list --query \"logo\" --sort-field uploadedAt --sort-direction desc\n\n\
        Response shape:\n\
        {\"data\": [{\"id\": \"...\", \"mimeType\": \"image/png\", \"title\": \"...\",\n\
          \"resolutions\": {\"original\": {\"url\": \"...\"}}}], \"cursor\": \"...\"}")]
    List {
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        query: Option<String>,
        #[arg(long, help = "Filter by MIME type (e.g., image/png, video/mp4)")]
        mime_type: Option<String>,
        #[arg(long)]
        sort_field: Option<String>,
        #[arg(long)]
        sort_direction: Option<String>,
    },
    /// Delete a media file
    Delete {
        /// Media ID
        id: String,
    },
}

pub async fn handle(cmd: MediaCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        MediaCommand::Upload { files } => {
            let mut file_data: Vec<(String, Vec<u8>, String)> = Vec::new();

            for path_str in &files {
                let path = std::path::Path::new(path_str);
                if !path.exists() {
                    return Err(CliError::InvalidInput(format!(
                        "File not found: {path_str}"
                    )));
                }

                let data = std::fs::read(path)?;
                let filename = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "file".to_string());

                let mime = mime_from_ext(path.extension().and_then(|e| e.to_str()));
                file_data.push((filename, data, mime));
            }

            let result = client
                .upload(&format!("/v1/businesses/{biz_id}/media"), file_data)
                .await?;
            crate::output::print_output(&result, format);
        }
        MediaCommand::List {
            limit,
            cursor,
            query,
            mime_type,
            sort_field,
            sort_direction,
        } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }
            if let Some(ref q) = query {
                params.push(("query", q.clone()));
            }
            if let Some(ref m) = mime_type {
                params.push(("mimeType", m.clone()));
            }
            if let Some(ref sf) = sort_field {
                params.push(("sortField", sf.clone()));
            }
            if let Some(ref sd) = sort_direction {
                params.push(("sortDirection", sd.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/media"), &params_ref)
                .await?;
            crate::output::print_output(&result, format);
        }
        MediaCommand::Delete { id } => {
            let result = client
                .delete(&format!("/v1/businesses/{biz_id}/media/{id}"))
                .await?;
            crate::output::print_output(&result, format);
            crate::output::print_success("Media deleted");
        }
    }
    Ok(())
}

fn mime_from_ext(ext: Option<&str>) -> String {
    match ext.map(|e| e.to_lowercase()).as_deref() {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("pdf") => "application/pdf",
        Some("json") => "application/json",
        Some("csv") => "text/csv",
        Some("txt") => "text/plain",
        Some("html") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("zip") => "application/zip",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("avif") => "image/avif",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
    .to_string()
}
