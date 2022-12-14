use curl::easy::Easy;

fn create_fake_headers(accept: &str) -> anyhow::Result<curl::easy::List> {
    let mut headers: curl::easy::List = curl::easy::List::new();
    headers.append("User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:105.0) Gecko/20100101 Firefox/105.0")?;
    headers.append(format!("Accept: {}", accept).as_str())?;
    headers.append("Accept-Language: en-US,en;q=0.5")?;
    headers.append("Accept-Encoding: identity")?;
    headers.append("DNT: 1")?;
    headers.append("Connection: keep-alive")?;
    headers.append("Upgrade-Insecure-Requests: 1")?;
    headers.append("Sec-Fetch-Dest: document")?;
    headers.append("Sec-Fetch-Mode: navigate")?;
    headers.append("Sec-Fetch-Site: cross-site")?;
    headers.append("Pragma: no-cache")?;
    headers.append("Cache-Control: no-cache")?;
    headers.append("TE: trailers")?;
    Ok(headers)
}

pub fn get_with_curl(easy: &mut Easy, accept: &str, url: &str) -> anyhow::Result<Vec<u8>> {
    easy.reset();
    easy.url(url)?;
    let headers = create_fake_headers(accept)?;
    easy.http_headers(headers)?;

    let mut dst = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            dst.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    Ok(dst)
}

pub fn post_with_curl(easy: &mut Easy, url: &str, post_data: &str) -> anyhow::Result<Vec<u8>> {
    easy.reset();
    easy.url(url)?;
    let mut headers = create_fake_headers(
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8",
    )?;
    headers.append("Content-Type: application/json")?;
    easy.http_headers(headers)?;

    easy.post_fields_copy(post_data.as_bytes())?;

    let mut dst = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            dst.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }

    Ok(dst)
}
