using System;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using MediatR;
using Microsoft.Extensions.Caching.Distributed;

namespace Tcproxy.Application.Requests.StartChallengeRequest;

public record StartChallengeResponse(
    Guid ChallengeId);

public record StartChallengeRequest(
    string CallbackUrl,
    uint Nonce) : IRequest<StartChallengeResponse>;

public record ChallengeRecord(
    Guid ChallengeId,
    string CallbackUrl,
    uint Nonce);

public class StartChallengeRequestHandler : IRequestHandler<StartChallengeRequest, StartChallengeResponse>
{
    private readonly IDistributedCache _distributedCache;

    public StartChallengeRequestHandler(IDistributedCache distributedCache)
    {
        _distributedCache = distributedCache;
    }

    public async Task<StartChallengeResponse> Handle(StartChallengeRequest request, CancellationToken cancellationToken)
    {
        var challengeRecord = new ChallengeRecord(Guid.NewGuid(), request.CallbackUrl, request.Nonce);
        var serializedObject = Encoding.UTF8.GetBytes(JsonSerializer.Serialize(challengeRecord));

        await _distributedCache.SetAsync(
            challengeRecord.ChallengeId.ToString(),
            serializedObject,
            cancellationToken);

        return new StartChallengeResponse(challengeRecord.ChallengeId);
    }
}