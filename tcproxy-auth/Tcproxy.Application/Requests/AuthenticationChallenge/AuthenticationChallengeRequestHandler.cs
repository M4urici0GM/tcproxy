using System;
using System.IO;
using System.Security.Authentication;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using MediatR;
using Microsoft.Extensions.Caching.Distributed;
using Tcproxy.Application.Requests.StartChallengeRequest;
using tcproxy.core;
using Tcproxy.Persistence.Repositories;

namespace Tcproxy.Application.Requests.AuthenticationChallenge;

public class AuthenticationChallengeRequestHandler
    : IRequestHandler<AuthenticationChallengeRequest, AuthenticationChallengeResponse>
{
    private readonly IUserRepository _userRepository;
    private readonly IDistributedCache _distributedCache;

    public AuthenticationChallengeRequestHandler(IUserRepository userRepository, IDistributedCache distributedCache)
    {
        _userRepository = userRepository;
        _distributedCache = distributedCache;
    }

    public async Task<AuthenticationChallengeResponse> Handle(
        AuthenticationChallengeRequest request,
        CancellationToken cancellationToken)
    {
        await ValidateChallenge(request.ChallengeId, cancellationToken);
        var userOption = await _userRepository.FindByEmailAsync(request.Email, cancellationToken);
        if (userOption.IsNone())
        {
            return new AuthenticationChallengeResponse(request.Email, null, null);
        }

        var user = userOption.Unwrap();
        return new AuthenticationChallengeResponse(user.Email, user.ProfilePicture, user.Name);
    }

    private async ValueTask ValidateChallenge(Guid challengeId, CancellationToken cancellationToken)
    {
        var challenge = await GetChallengeAsync(challengeId, cancellationToken);
        if (challenge.IsNone())
        {
            throw new InvalidCredentialException("invalid challenge-id");
        }
    }

    private async Task<Option<ChallengeRecord>> GetChallengeAsync(Guid challengeId, CancellationToken cancellationToken)
    {
        var challengeIdStr = challengeId.ToString();
        var challengeRecordBuff = await _distributedCache.GetAsync(challengeIdStr, cancellationToken);
        if (challengeRecordBuff == null)
        {
            return Option<ChallengeRecord>.From(null);
        }
        
        var stream = new MemoryStream(challengeRecordBuff);
        var challengeRecord = await JsonSerializer.DeserializeAsync<ChallengeRecord>(
            stream,
            cancellationToken: cancellationToken);
        
        return Option<ChallengeRecord>.From(challengeRecord);
    }
}