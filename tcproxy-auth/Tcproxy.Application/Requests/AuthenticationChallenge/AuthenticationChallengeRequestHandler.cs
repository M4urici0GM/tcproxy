using System.Threading;
using System.Threading.Tasks;
using MediatR;
using Tcproxy.Persistence.Repositories;

namespace Tcproxy.Application.Requests.AuthenticationChallenge;

public class AuthenticationChallengeRequestHandler
    : IRequestHandler<AuthenticationChallengeRequest, AuthenticationChallengeResponse>
{
    private readonly IUserRepository _userRepository;

    public AuthenticationChallengeRequestHandler(IUserRepository userRepository)
    {
        _userRepository = userRepository;
    }

    public async Task<AuthenticationChallengeResponse> Handle(
        AuthenticationChallengeRequest request,
        CancellationToken cancellationToken)
    {
        var userOption = await _userRepository.FindByEmailAsync(request.Email, cancellationToken);
        if (userOption.IsNone())
        {
            return new AuthenticationChallengeResponse(request.Email, null, null);
        }

        var user = userOption.Unwrap();
        return new AuthenticationChallengeResponse(user.Email, user.ProfilePicture, user.Name);
    }
}