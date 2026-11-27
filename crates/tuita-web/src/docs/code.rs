//! Code samples for the developer docs. They are code, not
//! interface text, so they stay in English.

/// `GET /me` with a token.
#[must_use]
pub(crate) fn curl_me(base: &str) -> String {
    format!("curl {base}/api/v1/me \\\n  -H \"Authorization: Bearer tuita_your_token\"")
}

/// A post from a script.
#[must_use]
pub(crate) fn curl_post(base: &str) -> String {
    format!(
        r#"curl {base}/api/v1/posts \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{{"text": "Hello from my bot", "cw": "", "sensitive": false, "mediaIds": []}}'"#
    )
}

/// What the event stream looks like on the wire.
pub(crate) const SSE_SAMPLE: &str = r#"event: notification
data: {"type":"notification","data":{"kind":"mention","actor":{"handle":"ana"},"post":{"id":"0193…","text":"@mybot hi!"}}}"#;

/// A bot that answers mentions, in Node.
#[must_use]
pub(crate) fn node_bot(base: &str) -> String {
    const BODY: &str = r"const { INSTANCE, TOKEN } = process.env;
const headers = { Authorization: `Bearer ${TOKEN}`, 'Content-Type': 'application/json' };

async function reply(post, handle) {
  const text = `@${handle} you said: ${post.text.replace(/@\w+/g, '').trim()}`;
  await fetch(`${INSTANCE}/api/v1/posts`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ text, cw: '', sensitive: false, replyTo: post.id, mediaIds: [] })
  });
}

for (;;) {
  const res = await fetch(`${INSTANCE}/api/v1/stream`, { headers });
  let buffer = '';
  for await (const chunk of res.body.pipeThrough(new TextDecoderStream())) {
    buffer += chunk;
    let end;
    while ((end = buffer.indexOf('\n\n')) >= 0) {
      const block = buffer.slice(0, end);
      buffer = buffer.slice(end + 2);
      const event = /^event: (.*)$/m.exec(block)?.[1];
      const data = /^data: (.*)$/m.exec(block)?.[1];
      if (event !== 'notification' || !data) continue;
      const { data: n } = JSON.parse(data);
      if (n.kind === 'mention' && n.post) await reply(n.post, n.actor.handle);
    }
  }
  await new Promise((r) => setTimeout(r, 5000)); // reconnect
}";
    format!("// Run with: INSTANCE={base} TOKEN=tuita_… node echo-bot.mjs\n{BODY}")
}

/// The same bot in Python.
pub(crate) const PYTHON_BOT: &str = r#"import json, os, requests
base, token = os.environ["INSTANCE"], os.environ["TOKEN"]
h = {"Authorization": f"Bearer {token}"}
with requests.get(f"{base}/api/v1/stream", headers=h, stream=True) as r:
    event = None
    for line in r.iter_lines(decode_unicode=True):
        if line.startswith("event: "):
            event = line[7:]
        elif line.startswith("data: ") and event == "notification":
            n = json.loads(line[6:])["data"]
            if n["kind"] == "mention" and n.get("post"):
                requests.post(f"{base}/api/v1/posts", headers=h, json={
                    "text": f"@{n['actor']['handle']} hello!", "cw": "",
                    "sensitive": False, "replyTo": n["post"]["id"], "mediaIds": []})"#;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn the_curl_samples_name_the_instance() {
        assert_eq!(
            curl_me("https://tuita.test"),
            "curl https://tuita.test/api/v1/me \\\n  -H \"Authorization: Bearer tuita_your_token\""
        );
        let post = curl_post("https://tuita.test");
        assert!(post.starts_with("curl https://tuita.test/api/v1/posts \\\n"));
        assert!(post.ends_with(r#""mediaIds": []}'"#), "{post}");
    }

    #[test]
    fn the_node_bot_keeps_its_template_literals() {
        let bot = node_bot("https://tuita.test");
        assert!(bot.starts_with(
            "// Run with: INSTANCE=https://tuita.test TOKEN=tuita_… node echo-bot.mjs\nconst {"
        ));
        assert!(bot.contains("`Bearer ${TOKEN}`"));
        assert!(bot.contains(r"/@\w+/g"));
        assert!(bot.contains(r"indexOf('\n\n')"));
    }
}
