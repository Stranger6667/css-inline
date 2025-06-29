package org.cssinline;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.IOException;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;
import org.openjdk.jmh.annotations.*;
import org.openjdk.jmh.runner.Runner;
import org.openjdk.jmh.runner.RunnerException;
import org.openjdk.jmh.runner.options.Options;
import org.openjdk.jmh.runner.options.OptionsBuilder;

@State(Scope.Benchmark)
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.MICROSECONDS)
@Fork(value = 1, jvmArgs = {"-server"})
@Warmup(iterations = 1, time = 10, timeUnit = TimeUnit.SECONDS)
@Measurement(iterations = 3, time = 10, timeUnit = TimeUnit.SECONDS)
public class CSSInlineBench {

	@Param({"simple", "merging", "double_quotes", "big_email_1", "big_email_2", "big_page"})
	public String name;

	private static Map<String, String> cases;

	private CssInlineConfig cfg;

	private CSSBoxInliner cssBoxInliner;

	@Setup(Level.Trial)
	public void setup() throws IOException {
		cfg = new CssInlineConfig.Builder().build();
		cssBoxInliner = new CSSBoxInliner();

		if (cases == null) {
			Path path = Paths.get("").resolve("../../benchmarks/benchmarks.json");

			List<Map<String, String>> list = new ObjectMapper().readValue(path.toFile(),
					new TypeReference<List<Map<String, String>>>() {
					});

			cases = new LinkedHashMap<>();
			for (Map<String, String> entry : list) {
				cases.put(entry.get("name"), entry.get("html"));
			}
		}
	}

	@Benchmark
	public String benchCSSInline() {
		return CssInline.inline(cases.get(name), cfg);
	}

	@Benchmark
	public String benchCSSBox() {
		try {
			return cssBoxInliner.inline(cases.get(name));
		} catch (Exception e) {
			throw new RuntimeException("CSSBox benchmark failed", e);
		}
	}

	public static void main(String[] args) throws RunnerException {
		Options opt = new OptionsBuilder().include(CSSInlineBench.class.getSimpleName()).build();
		new Runner(opt).run();
	}
}
