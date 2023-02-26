using System.Threading;
using System.Threading.Tasks;
using FluentValidation;
using MediatR;

namespace Tcproxy.Api.Application.Behaviours;

/// <summary>
/// Pipeline to validate request before executing actual RequestHandler
/// </summary>
/// <typeparam name="TRequest"></typeparam>
/// <typeparam name="TResponse"></typeparam>
public class ValidationBehavior<TRequest, TResponse> : IPipelineBehavior<TRequest, TResponse> where TRequest : IRequest<TResponse>
{
    private readonly bool _hasValidator;
    private readonly IValidator<TRequest> _requestValidators;

    public ValidationBehavior()
    {
        _hasValidator = false;
    }
    
    public ValidationBehavior(IValidator<TRequest> requestValidators)
    {
        _requestValidators = requestValidators;
        _hasValidator = true;
    }
    
    /// <summary>
    /// Executes before any other commands, validating the command if a validator is available on
    /// dependency injection container.
    /// </summary>
    /// <param name="request"></param>
    /// <param name="cancellationToken"></param>
    /// <param name="next"></param>
    /// <returns></returns>
    /// <exception cref="System.ComponentModel.DataAnnotations.ValidationException"></exception>
    public async Task<TResponse> Handle(TRequest request, RequestHandlerDelegate<TResponse> next, CancellationToken cancellationToken)
    {
        if (!_hasValidator)
            return await next();

        var validationResult = await _requestValidators.ValidateAsync(request, cancellationToken);
        if (!validationResult.IsValid)
            throw new ValidationException(
                $"Command validation for type {typeof(TRequest).Name}",
                validationResult.Errors);

        return await next();
    }
}