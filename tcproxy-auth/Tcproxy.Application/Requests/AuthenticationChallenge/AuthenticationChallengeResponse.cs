namespace Tcproxy.Application.Requests.AuthenticationChallenge;

public record AuthenticationChallengeResponse(
    string UserEmail,
    string ProfilePicture,
    string UserName);