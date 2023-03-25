using MediatR;
using Tcproxy.Application.Responses;

namespace Tcproxy.Application.Requests.CreateUser;

/// <summary>
/// Request object used for creating new User.
/// </summary>
public class CreateUserRequest : IRequest<UserResponse>
{
    public string FirstName { get; set; } = string.Empty;
    public string LastName { get; set; } = string.Empty;
    public string Email { get; set; } = string.Empty;
    public string Password { get; set; } = string.Empty;
}