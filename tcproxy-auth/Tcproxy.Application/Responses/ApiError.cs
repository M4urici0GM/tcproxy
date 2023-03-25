using System.Collections.Generic;

namespace Tcproxy.Application.Responses;

public class ApiError<T>
{
    public int StatusCode { get; set; }
    public T Content { get; set; }
    public IEnumerable<ValidationErrorDto> ValidationErrors { get; set; }
}