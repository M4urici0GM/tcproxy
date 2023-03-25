using System.Threading;
using System.Threading.Tasks;
using MediatR;
using Microsoft.AspNetCore.Mvc;
using Tcproxy.Application.Requests.AuthenticationChallenge;
using Tcproxy.Application.Requests.StartChallengeRequest;

namespace tcproxy.api.Controllers;

[ApiController, Route("/v1/auth")]
public class AuthenticationController : ControllerBase
{
    private readonly IMediator _mediator;

    public AuthenticationController(IMediator mediator)
    {
        _mediator = mediator;
    }

    [HttpPost("start-challenge")]
    public async Task<IActionResult> StartChallengeAsync(
        [FromBody]StartChallengeRequest request,
        CancellationToken cancellationToken)
    {
        var response = await _mediator.Send(request, cancellationToken);
        return Ok(response);
    }

    [HttpGet("challenge")]
    public async Task<IActionResult> ChallengeAsync(
        [FromQuery] AuthenticationChallengeRequest request,
        CancellationToken cancellationToken)
    {
        var response = await _mediator.Send(request, cancellationToken);
        return Ok(response);
    }
}