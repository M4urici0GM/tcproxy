using System;
using MediatR;

namespace Tcproxy.Application.Requests.AuthenticationChallenge;

public record AuthenticationChallengeRequest(
    string Email,
    Guid ChallengeId) : IRequest<AuthenticationChallengeResponse>;