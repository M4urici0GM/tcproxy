namespace Tcproxy.Application.Responses;

/// <summary>
/// Represents a validation error.
/// </summary>
/// <param name="Property"></param>
/// <param name="Message"></param>
public record ValidationErrorDto(
    string Property,
    string Message);