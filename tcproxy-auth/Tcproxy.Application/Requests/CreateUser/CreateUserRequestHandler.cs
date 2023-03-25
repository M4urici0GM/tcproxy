using System.Threading;
using System.Threading.Tasks;
using MediatR;
using Tcproxy.Application.Responses;
using Tcproxy.Core.Entities;
using Tcproxy.Core.Exceptions;
using Tcproxy.Persistence.Repositories;

namespace Tcproxy.Application.Requests.CreateUser;

/// <summary>
/// Handles CreateUserRequest
/// </summary>
public class CreateUserRequestHandler : IRequestHandler<CreateUserRequest, UserResponse>
{
    private readonly IUserRepository _userRepository;

    public CreateUserRequestHandler(IUserRepository userRepository)
    {
        _userRepository = userRepository;
    }

    /// <summary>
    /// Tries to create a new user.
    /// </summary>
    /// <param name="request"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    /// <exception cref="EntityAlreadyExists">When user already exists with given email.</exception>
    public async Task<UserResponse> Handle(CreateUserRequest request, CancellationToken cancellationToken)
    {
        var userExists = await _userRepository.UserExistsByEmailAsync(request.Email, cancellationToken);
        if (userExists)
        {
            throw new EntityAlreadyExists(nameof(User.Email), nameof(User), request.Email);
        }
        
        var passwordHash = BCrypt.Net.BCrypt.HashPassword(request.Password);
        var user = new User
        {
            Name = $"{request.FirstName} {request.LastName}",
            Email = request.Email,
            PasswordHash = passwordHash,
        };

        var persistedUser = await _userRepository.InsertOneAsync(user, cancellationToken);
        return new UserResponse(persistedUser.Id, persistedUser.Name, persistedUser.Email, persistedUser.CreatedAt);
    }
}