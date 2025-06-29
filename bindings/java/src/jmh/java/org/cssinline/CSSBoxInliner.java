package org.cssinline;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import org.fit.cssbox.css.CSSNorm;
import org.fit.cssbox.css.DOMAnalyzer;
import org.fit.cssbox.css.NormalOutput;
import org.fit.cssbox.css.Output;
import org.fit.cssbox.io.DOMSource;
import org.fit.cssbox.io.DefaultDOMSource;
import org.fit.cssbox.io.DefaultDocumentSource;
import org.fit.cssbox.io.DocumentSource;
import org.w3c.dom.Document;

public class CSSBoxInliner {

	public String inline(String html) throws Exception {
		String dataUrl = "data:text/html;charset=utf-8," + java.net.URLEncoder.encode(html, "UTF-8");
		DocumentSource docSource = new DefaultDocumentSource(dataUrl);

		try {
			DOMSource parser = new DefaultDOMSource(docSource);
			Document doc = parser.parse();

			DOMAnalyzer da = new DOMAnalyzer(doc, docSource.getURL());
			da.attributesToStyles();
			da.addStyleSheet(null, CSSNorm.userStyleSheet(), DOMAnalyzer.Origin.AGENT);
			da.getStyleSheets();

			da.stylesToDomInherited();

			ByteArrayOutputStream baos = new ByteArrayOutputStream();
			PrintStream ps = new PrintStream(baos, true, "UTF-8");
			Output out = new NormalOutput(doc);
			out.dumpTo(ps);
			ps.close();

			return baos.toString("UTF-8");

		} finally {
			docSource.close();
		}
	}
}
