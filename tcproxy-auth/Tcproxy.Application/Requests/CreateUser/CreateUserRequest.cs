using MediatR;
using Tcproxy.Application.Interfaces;
using Tcproxy.Application.Responses;

namespace Tcproxy.Application.Requests.CreateUser;

public class CreateUserRequest : IRequest<UserResponse>
{
    public string Name { get; set; } = string.Empty;
    public string Email { get; set; } = string.Empty;
    public string Password { get; set; } = string.Empty;
}