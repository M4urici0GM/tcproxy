using MediatR;
using Microsoft.AspNetCore.Http;

namespace Tcproxy.Application.Interfaces;

public interface IHttpRequest : IRequest<IResult>
{
    
}