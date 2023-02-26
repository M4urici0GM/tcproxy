using System;

namespace Tcproxy.Application.Responses;

public record UserResponse(
    Guid Id,
    string Name,
    string Email,
    DateTime CreatedAt);