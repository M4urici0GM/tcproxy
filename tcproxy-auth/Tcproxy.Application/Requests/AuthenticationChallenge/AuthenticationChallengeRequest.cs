using MediatR;

namespace Tcproxy.Application.Requests.AuthenticationChallenge;

public record AuthenticationChallengeRequest(
    string Email) : IRequest<AuthenticationChallengeResponse>;