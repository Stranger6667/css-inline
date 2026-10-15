package org.cssinline;

/**
 * Exception thrown when CSS inlining operations fail.
 *
 * This runtime exception is thrown when errors occur during CSS processing,
 * HTML parsing, or stylesheet loading.
 */
public class CssInlineException extends RuntimeException {

	/**
	 * Creates a new exception with the specified error message.
	 *
	 * @param message
	 *            the error message describing what went wrong
	 */
	public CssInlineException(String message) {
		super(message);
	}

	/**
	 * Creates a new exception with the specified error message and underlying
	 * cause.
	 *
	 * @param message
	 *            the error message describing what went wrong
	 * @param cause
	 *            the underlying exception that caused this error
	 */
	public CssInlineException(String message, Throwable cause) {
		super(message, cause);
	}
}
