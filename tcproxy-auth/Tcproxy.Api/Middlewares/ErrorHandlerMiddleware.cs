using System;
using System.Linq;
using System.Net;
using System.Threading.Tasks;
using FluentValidation;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Hosting;
using Tcproxy.Application.Responses;
using Tcproxy.Core.Exceptions;

namespace tcproxy.api.Middlewares;

/// <summary>
/// Handles all app base errors.
/// </summary>
public class ErrorHandlerMiddleware
{
    private readonly RequestDelegate _requestDelegate;
    private readonly IHostEnvironment _hostEnvironment;

    public ErrorHandlerMiddleware(RequestDelegate requestDelegate, IHostEnvironment hostEnvironment)
    {
        _requestDelegate = requestDelegate;
        _hostEnvironment = hostEnvironment;
    }

    public async Task Invoke(HttpContext httpContext)
    {
        try
        {
            await _requestDelegate.Invoke(httpContext);
        }
        // TODO: refactor this into a simpler code.
        catch (ValidationException e)
        {
            await WriteCustomResponse(httpContext, new ApiError<string>()
            {
                Content = "One or more validation errors occurred",
                StatusCode = (int)HttpStatusCode.BadRequest,
                ValidationErrors = e.Errors
                    .Select(error => new ValidationErrorDto(
                        error.PropertyName,
                        error.ErrorMessage)),
            });
        }
        catch (EntityAlreadyExists e)
        {
            await WriteCustomResponse(httpContext, new ApiError<string>()
            {
                Content = e.Message,
                StatusCode = (int)HttpStatusCode.Conflict
            });
        }
        catch (InvalidCredentialsException e)
        {
            await WriteCustomResponse(httpContext,new ApiError<string>()
            {
                StatusCode = (int)HttpStatusCode.Unauthorized,
                Content = e.Message,
            });
        }
        catch (NotFoundException e)
        {
            await WriteCustomResponse(httpContext,  new ApiError<string>()
            {
                StatusCode = (int)HttpStatusCode.Conflict,
                Content = e.Message,
            });
        }
        catch (Exception e)
        {
            await WriteCustomResponse(httpContext, new ApiError<string>()
            {
                StatusCode = (int)HttpStatusCode.Conflict,
                Content = _hostEnvironment.IsProduction()
                    ? "Internal Server Error"
                    : e.Message,
            });
        }
    }

    /// <summary>
    /// Writes given object as Json to response body.
    /// </summary>
    /// <param name="context"></param>
    /// <param name="content"></param>
    /// <typeparam name="T"></typeparam>
    private static async Task WriteCustomResponse<T>(HttpContext context, ApiError<T> content)
    {
        context.Response.ContentType = "application/json";
        context.Response.StatusCode = content.StatusCode;
        await context.Response.WriteAsJsonAsync(content);
    }
}