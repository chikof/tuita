//! Reference queries, rendered as
//! `terms=..;ex=..;from=..;to=..;has=..;since=..;until=..;ops=..` with lists joined by `|`.

pub(super) const CASES: &[(&str, &str)] = &[
    ("", "terms=;ex=;from=;to=;has=;since=;until=;ops=0"),
    ("   ", "terms=;ex=;from=;to=;has=;since=;until=;ops=0"),
    (
        "hello",
        "terms=hello;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "hello world",
        "terms=hello|world;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "  hello   world  ",
        "terms=hello|world;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "\"exact phrase\"",
        "terms=exact phrase;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "-\"exact phrase\"",
        "terms=;ex=exact phrase;from=;to=;has=;since=;until=;ops=1",
    ),
    ("-word", "terms=;ex=word;from=;to=;has=;since=;until=;ops=1"),
    (
        "a -b c",
        "terms=a|c;ex=b;from=;to=;has=;since=;until=;ops=1",
    ),
    ("-", "terms=-;ex=;from=;to=;has=;since=;until=;ops=0"),
    ("--", "terms=;ex=-;from=;to=;has=;since=;until=;ops=1"),
    ("--x", "terms=;ex=-x;from=;to=;has=;since=;until=;ops=1"),
    ("\"", "terms=;ex=;from=;to=;has=;since=;until=;ops=0"),
    ("\"\"", "terms=;ex=;from=;to=;has=;since=;until=;ops=0"),
    ("\"a", "terms=a;ex=;from=;to=;has=;since=;until=;ops=0"),
    ("a\"", "terms=a;ex=;from=;to=;has=;since=;until=;ops=0"),
    ("\"a\"b", "terms=a|b;ex=;from=;to=;has=;since=;until=;ops=0"),
    (
        "a\"b c\"d",
        "terms=a\"b|c\"d;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "from:ana",
        "terms=;ex=;from=ana;to=;has=;since=;until=;ops=1",
    ),
    (
        "FROM:Ana",
        "terms=;ex=;from=Ana;to=;has=;since=;until=;ops=1",
    ),
    (
        "from:@ana",
        "terms=;ex=;from=ana;to=;has=;since=;until=;ops=1",
    ),
    (
        "from:",
        "terms=from:;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    ("from:@", "terms=;ex=;from=;to=;has=;since=;until=;ops=1"),
    ("to:bo", "terms=;ex=;from=;to=bo;has=;since=;until=;ops=1"),
    ("to:@bo", "terms=;ex=;from=;to=bo;has=;since=;until=;ops=1"),
    ("TO:Bo", "terms=;ex=;from=;to=Bo;has=;since=;until=;ops=1"),
    (
        "from:ana from:bo",
        "terms=;ex=;from=ana|bo;to=;has=;since=;until=;ops=1",
    ),
    (
        "has:media",
        "terms=;ex=;from=;to=;has=media;since=;until=;ops=1",
    ),
    (
        "has:image",
        "terms=;ex=;from=;to=;has=image;since=;until=;ops=1",
    ),
    (
        "has:images",
        "terms=;ex=;from=;to=;has=image;since=;until=;ops=1",
    ),
    (
        "has:video",
        "terms=;ex=;from=;to=;has=video;since=;until=;ops=1",
    ),
    (
        "has:videos",
        "terms=;ex=;from=;to=;has=video;since=;until=;ops=1",
    ),
    (
        "has:poll",
        "terms=;ex=;from=;to=;has=poll;since=;until=;ops=1",
    ),
    (
        "has:links",
        "terms=;ex=;from=;to=;has=links;since=;until=;ops=1",
    ),
    (
        "has:link",
        "terms=;ex=;from=;to=;has=links;since=;until=;ops=1",
    ),
    (
        "has:LINKS",
        "terms=;ex=;from=;to=;has=links;since=;until=;ops=1",
    ),
    ("has:cats", "terms=;ex=;from=;to=;has=;since=;until=;ops=1"),
    ("has:", "terms=has:;ex=;from=;to=;has=;since=;until=;ops=0"),
    (
        "has:media has:poll",
        "terms=;ex=;from=;to=;has=media|poll;since=;until=;ops=1",
    ),
    (
        "since:2024-01-31",
        "terms=;ex=;from=;to=;has=;since=2024-01-31;until=;ops=1",
    ),
    (
        "until:2024-01-31",
        "terms=;ex=;from=;to=;has=;since=;until=2024-01-31;ops=1",
    ),
    (
        "since:2024-01-01 until:2024-12-31",
        "terms=;ex=;from=;to=;has=;since=2024-01-01;until=2024-12-31;ops=1",
    ),
    (
        "since:2024-03-01 since:2024-01-01",
        "terms=;ex=;from=;to=;has=;since=2024-03-01;until=;ops=1",
    ),
    (
        "until:2024-03-01 until:2024-01-01",
        "terms=;ex=;from=;to=;has=;since=;until=2024-01-01;ops=1",
    ),
    (
        "since:2024-1-5",
        "terms=;ex=;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "since:soon",
        "terms=;ex=;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "since:2024-13-01",
        "terms=;ex=;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "since:2024-00-10",
        "terms=;ex=;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "since:2024-02-29",
        "terms=;ex=;from=;to=;has=;since=2024-02-29;until=;ops=1",
    ),
    (
        "until:2024-12-31",
        "terms=;ex=;from=;to=;has=;since=;until=2024-12-31;ops=1",
    ),
    (
        "since:2024-01-31T10:00",
        "terms=;ex=;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "since:20240131",
        "terms=;ex=;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "-from:ana",
        "terms=;ex=from:ana;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "-has:media",
        "terms=;ex=has:media;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "-since:2024-01-01",
        "terms=;ex=since:2024-01-01;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "-\"from:ana\"",
        "terms=;ex=from:ana;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "\"from:ana\"",
        "terms=from:ana;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "from:ana \"has:media\"",
        "terms=has:media;ex=;from=ana;to=;has=;since=;until=;ops=1",
    ),
    (
        "cats -dogs \"good boy\" from:@ana has:media since:2024-01-31",
        "terms=cats|good boy;ex=dogs;from=ana;to=;has=media;since=2024-01-31;until=;ops=1",
    ),
    (
        "hello \"a b\" -no -\"c d\" from:ana to:@bo has:media has:poll has:links since:2024-01-31 until:2024-02-01",
        "terms=hello|a b;ex=no|c d;from=ana;to=bo;has=media|poll|links;since=2024-01-31;until=;ops=1",
    ),
    (
        "http://x.y",
        "terms=http://x.y;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    ("a:b", "terms=a:b;ex=;from=;to=;has=;since=;until=;ops=0"),
    (
        "from:a:b",
        "terms=;ex=;from=a:b;to=;has=;since=;until=;ops=1",
    ),
    ("from::x", "terms=;ex=;from=:x;to=;has=;since=;until=;ops=1"),
    (
        "from:\"x y\"",
        "terms=y;ex=;from=\"x;to=;has=;since=;until=;ops=1",
    ),
    (
        "from:x y",
        "terms=y;ex=;from=x;to=;has=;since=;until=;ops=1",
    ),
    ("#tag", "terms=#tag;ex=;from=;to=;has=;since=;until=;ops=0"),
    ("@ana", "terms=@ana;ex=;from=;to=;has=;since=;until=;ops=0"),
    (
        "\"#tag\" -\"@ana\"",
        "terms=#tag;ex=@ana;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "100% _ \\",
        "terms=100%|_|\\;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "\u{fc}n\u{ef} c\u{f6}d\u{e9}",
        "terms=\u{fc}n\u{ef}|c\u{f6}d\u{e9};ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "a\u{9}b\u{a}c",
        "terms=a|b|c;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "\"a\u{a}b\"",
        "terms=a\u{a}b;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "-\"a b\" \"c",
        "terms=c;ex=a b;from=;to=;has=;since=;until=;ops=1",
    ),
    (
        "x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x x ",
        "terms=x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x|x;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa tail",
        "terms=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa|t;ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}",
        "terms=\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9}\u{e9};ex=;from=;to=;has=;since=;until=;ops=0",
    ),
    (
        "from:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "terms=;ex=;from=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa;to=;has=;since=;until=;ops=1",
    ),
];
