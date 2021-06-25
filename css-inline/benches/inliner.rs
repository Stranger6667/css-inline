use criterion::{black_box, criterion_group, criterion_main, Criterion};
use css_inline::{inline, CSSInliner, InlineError};

fn simple(c: &mut Criterion) {
    let html = black_box(
        r#"<html>
<head>
    <title>Test</title>
    <style>
        h1, h2 { color:blue; }
        strong { text-decoration:none }
        p { font-size:2px }
        p.footer { font-size: 1px}
    </style>
</head>
<body>
    <h1>Big Text</h1>
    <p>
        <strong>Solid</strong>
    </p>
    <p class="footer">Foot notes</p>
</body>
</html>"#,
    );
    c.bench_function("simple HTML", |b| b.iter(|| inline(html).unwrap()));
}

fn external(c: &mut Criterion) {
    let html = black_box(
        r#"<html>
<head><link href="benches/styles.css" rel="stylesheet" type="text/css"></head>
<body>
    <h1>Big Text</h1>
    <p>
        <strong>Solid</strong>
    </p>
    <p class="footer">Foot notes</p>
</body>
</html>"#,
    );
    c.bench_function("External CSS", |b| b.iter(|| inline(html).unwrap()));
}

fn error_formatting(c: &mut Criterion) {
    let error = black_box(InlineError::ParseError("Error description".into()));
    c.bench_function("parse error formatting", |b| b.iter(|| error.to_string()));
}

fn io_error_formatting(c: &mut Criterion) {
    let error = black_box(
        inline(
            r#"
<html>
<head><link href="unknown.css" rel="stylesheet" type="text/css"></head>
<body></body>
</html>"#,
        )
        .expect_err("It is an error"),
    );
    c.bench_function("io error formatting", |b| b.iter(|| error.to_string()));
}

fn network_error_formatting(c: &mut Criterion) {
    let error = black_box(
        inline(
            r#"
<html>
<head><link href="http://127.0.0.1:0/unknown.css" rel="stylesheet" type="text/css"></head>
<body></body>
</html>"#,
        )
        .expect_err("It is an error"),
    );
    c.bench_function("network error formatting", |b| b.iter(|| error.to_string()));
}

fn merging(c: &mut Criterion) {
    let html = black_box(
        r#"<html>
<head>
    <title>Test</title>
    <style>
        h1, h2 { color:blue; }
        strong { text-decoration:none }
        p { font-size:2px }
        p.footer { font-size: 1px}
    </style>
</head>
<body>
    <h1 style="background-color: black;">Big Text</h1>
    <p style="background-color: black;">
        <strong style="background-color: black;">Solid</strong>
    </p>
    <p class="footer" style="background-color: black;">Foot notes</p>
</body>
</html>"#,
    );
    c.bench_function("merging styles", |b| b.iter(|| inline(html).unwrap()));
}

fn removing_tags(c: &mut Criterion) {
    let html = black_box(
        r#"<html>
<head>
<style>
h1 {
    text-decoration: none;
}
</style>
<style>
.test-class {
        color: #ffffff;
}
a {
        color: #17bebb;
}
</style>
</head>
<body>
<a class="test-class" href="https://example.com">Test</a>
<h1>Test</h1>
</body>
</html>"#,
    );
    let inliner = CSSInliner::compact();
    c.bench_function("removing tags", |b| {
        b.iter(|| inliner.inline(html).unwrap())
    });
}

fn big_email(c: &mut Criterion) {
    let html = black_box(
        r##"<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN">
<html>
<head>
        <title> Newsletter</title>
	<meta name="viewport" content="width = 620" />
<style type="text/css">
img {margin:0}
a.bluelink:link,a.bluelink:visited,a.bluelink:active {color:#5b7ab3; text-decoration: none}
a.bluelink:hover {color:#5b7ab3; text-decoration: underline}
</style>
<style media="only screen and (max-device-width: 480px)" type="text/css">
* {line-height: normal !important; -webkit-text-size-adjust: 125%}
</style>
</head>

<body bgcolor="#FFFFFF" style="margin:0; padding:0">
<table width="100%" bgcolor="#FFFFFF" cellpadding="0" cellspacing="0" align="center" border="2">
	<tr>
		<td style="padding: 30px"><!--
--><table width="636" border="0" cellspacing="0" cellpadding="0" align="center">
	<tr>
		<td width="636"><img src="http://images.apple.com/data/retail/us/topcap.gif" border="0" alt="" width="636" height="62" style="display:block" /></td>
	</tr>
</table><!--
--><table width="636" border="1" cellspacing="0" cellpadding="0" align="center" bgcolor="#fffef6">
	<tr>
		<td width="59" valign="top" background="http://images.apple.com/data/retail/us/leftbg.gif"><img src="http://images.apple.com/data/retail/us/leftcap.gif" width="59" height="302" border="1" alt="" style="display:block" /></td>
		<td width="500" align="left" valign="top"><!--
--><table width="500" border="1" cellspacing="0" cellpadding="0">
	<tr>
		<td width="379" align="left" valign="top">
<div><img src="http://images.apple.com/data/retail/us/headline.gif" width="330" height="29" border="1" alt="Thanks for making a reservation." style="display:block" /></div>
		</td>
		<td width="21" align="right" valign="top">
<div><img src="http://images.apple.com/data/retail/us/applelogo.gif" width="21" height="25" border="1" alt="" style="display:block" /></div>
		</td>
	</tr>
</table><!--
--><table width="500" border="1" cellspacing="0" cellpadding="0">
	<tr>
		<td width="500" align="left" valign="top">
<div><img src="http://images.apple.com/data/retail/us/line.gif" width="500" height="36" border="0" alt="" style="display:block" /></div>
		</td>
	</tr>
</table><!--
--><table width="500" border="1" cellspacing="0" cellpadding="0">
	<tr>
		<td width="10" align="left" valign="top"></td>
		<td width="340" align="left" valign="top">


<div style="margin: 0; padding: 2px 10px 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color: #000000 !important; font-size:12px; line-height: 16px">Dear peter,</div>
<div style="margin: 0; padding: 12px 10px 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color: #000000 !important; font-size:12px; line-height: 16px">You are scheduled for a Genius Bar appointment.</div>
<div style="margin: 0; padding: 12px 10px 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color: #000000 !important; font-size:12px; line-height: 16px">Topic: <b>iPhone</b></div>
<div style="margin: 0; padding: 12px 10px 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color: #000000 !important; font-size:12px; line-height: 16px">Date: <b>Wednesday, Aug 26, 2009</b></div>
<div style="margin: 0; padding: 12px 10px 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color: #000000 !important; font-size:12px; line-height: 16px">Time: <b>11:10AM</b></div>
<div style="margin: 0; padding: 12px 10px 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color: #000000 !important; font-size:12px; line-height: 16px">Location: <b>Apple Store, Regent Street</b></div>
		</td>
		<td width="150" align="left" valign="top">
<div style="margin: 0; padding: 2px 0 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color:#808285; font-size:11px; line-height: 13px">Apple Store,</div>
<div style="margin: 0; padding: 0 0 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color:#808285; font-size:11px; line-height: 13px">Regent Street</div>
<div style="margin: 0; padding: 7px 0 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color:#808285; font-size:11px; line-height: 13px"><a href="http://concierge.apple.com/WebObjects/RRSServices.woa/wa/ics?id=ewoJInByaW1hcnlLZXkiID0gewoJCSJyZXNlcnZhdGlvbklEIiA9ICI1ODEyMDI2NCI7Cgl9OwoJImVudGl0eU5hbWUiID0gIlJlc2VydmF0aW9uIjsKfQ%3D%3D" style="font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; font-size:11px; color:#5b7ab3" class="bluelink">Add this to your calendar<img src="http://images.apple.com/data/retail/us/bluearrow.gif" width="8" height="8" border="0" alt="" style="display:inline; margin:0" /></a></div>
<div style="margin: 0; padding: 7px 0 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color:#808285; font-size:11px; line-height: 13px">If you are no longer able to attend this session, please <a href="http://concierge.apple.com/WebObjects/Concierge.woa/wa/cancelReservation?r=ewoJInByaW1hcnlLZXkiID0gewoJCSJyZXNlcnZhdGlvbklEIiA9ICI1ODEyMDI2NCI7Cgl9OwoJImVudGl0eU5hbWUiID0gIlJlc2VydmF0aW9uIjsKfQ%3D%3D" style="font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; font-size:11px; color:#5b7ab3" class="bluelink">cancel</a> or <a href="http://concierge.apple.com/WebObjects/Concierge.woa/wa/cancelReservation?r=ewoJInByaW1hcnlLZXkiID0gewoJCSJyZXNlcnZhdGlvbklEIiA9ICI1ODEyMDI2NCI7Cgl9OwoJImVudGl0eU5hbWUiID0gIlJlc2VydmF0aW9uIjsKfQ%3D%3D" style="font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; font-size:11px; color:#5b7ab3" class="bluelink">reschedule</a> your reservation.</div>
<div style="margin: 0; padding: 7px 0 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color:#808285; font-size:11px; line-height: 13px"><a href="http://www.apple.com/retail/../uk/retail/regentstreet/map" style="font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; font-size:11px; color:#5b7ab3" class="bluelink">Get directions to the store<img src="http://images.apple.com/data/retail/us/bluearrow.gif" width="8" height="8" border="0" alt="" style="display:inline; margin:0" /></a></div>
		</td>
	</tr>
</table><!--
--><table width="500" border="1" cellspacing="0" cellpadding="0">
	<tr>
    <td width="10"></td>
				<td width="490" align="left" valign="top">
				<br>
<div style="margin: 0; padding: 0 20px 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color: #000000 !important; font-size:12px; line-height: 16px">We look forward to seeing you.</div>
<div style="margin: 0; padding: 0 20px 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color: #000000 !important; font-size:12px; line-height: 16px">Your Apple Store team,</div>
<div style="margin: 0; padding: 0 20px 0 0; font-family: Lucida Grande, Arial, Helvetica, Geneva, Verdana, sans-serif; color: #000000 !important; font-size:12px; line-height: 16px">Regent Street</div>
		</td>
	</tr>
</table><!--
		--></td>
		<td width="59" valign="top" background="http://images.apple.com/data/retail/us/rightbg.gif"><img src="http://images.apple.com/data/retail/us/rightcap.gif" width="77" height="302" border="0" alt="" style="display:block" /></td>
	</tr>
</table><!--
--><table width="636" border="1" cellspacing="0" cellpadding="0" align="center">
	<tr>
		<td width="636"><img src="http://images.apple.com/data/retail/us/bottomcap.gif" border="0" alt="" width="636" height="62" style="display:block" /></td>
	</tr>
</table><!--
BEGIN FOOTER
--><table width="498" border="1" cellspacing="0" cellpadding="0" align="center">
	<tr>
		<td style="padding-top:22px">
<div style="font-family: Geneva, Verdana, Arial, Helvetica, sans-serif; font-size:9px; line-height: 12px; color:#b4b4b4">TM and copyright &copy; 2008 Apple Inc. 1 Infinite Loop, MS 303-3DM, Cupertino, CA 95014.</div>
<div style="font-family: Geneva, Verdana, Arial, Helvetica, sans-serif; font-size:9px; line-height: 12px; color:#b4b4b4"><a href="http://www.apple.com/legal/" style="font-family: Geneva, Verdana, Arial, Helvetica, sans-serif; font-size:9px;line-height: 12px; color:#b4b4b4; text-decoration:underline">All Rights Reserved</a> / <a href="http://www.apple.com/enews/subscribe/"  style="font-family: Geneva, Verdana, Arial, Helvetica, sans-serif; font-size:9px; line-height: 12px;color:#b4b4b4; text-decoration:underline">Keep Informed</a> / <a href="http://www.apple.com/legal/privacy/" style="font-family: Geneva, Verdana, Arial, Helvetica, sans-serif; font-size:9px; line-height: 12px; color:#b4b4b4; text-decoration:underline">Privacy Policy</a> / <a href="https://myinfo.apple.com/cgi-bin/WebObjects/MyInfo/" style="font-family: Geneva, Verdana, Arial, Helvetica, sans-serif; font-size:9px; line-height: 12px; color:#b4b4b4; text-decoration:underline">My Info</a></div>
		</td>
	</tr>
</table><!--
		--></td>
	</tr>
</table>
</body>
</html>"##,
    );
    c.bench_function("big email", |b| b.iter(|| inline(html).unwrap()));
}

criterion_group!(
    benches,
    simple,
    external,
    merging,
    removing_tags,
    big_email,
    error_formatting,
    io_error_formatting,
    network_error_formatting,
);
criterion_main!(benches);
