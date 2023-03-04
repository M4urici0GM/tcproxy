using System.Reflection;
using FluentValidation;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using tcproxy.api.Application.Behaviours;
using Tcproxy.Application.Mappings;
using Tcproxy.Application.Requests.CreateUser;
using tcproxy.core.Options;
using Tcproxy.Persistence.Context;
using Tcproxy.Persistence.Repositories;

namespace tcproxy.api.Application.Extensions;

public static class ServiceCollectionExtensions
{
    public static void AddEssentialServices(this IServiceCollection services)
    {
        services.AddAutoMapper(cfg => cfg.AddProfile<AutoMapperProfile>());
        services.AddMediatR(cfg =>
        {
            cfg.AddOpenBehavior(typeof(ValidationBehavior<,>));
            cfg.RegisterServicesFromAssemblies(
                Assembly.GetExecutingAssembly(),
                typeof(CreateUserRequest).Assembly);
        });
    }

    public static void AddPersistence(this IServiceCollection services)
    {
        services.AddSingleton<IMongodbContext, MongodbContext>();
        services.AddScoped<IUserRepository, UserRepository>();
    }

    public static void AddValidators(this IServiceCollection services)
    {
        services.AddScoped<IValidator<CreateUserRequest>, CreateUserRequestValidator>();
    }
    
    public static void AddAppOptions(this IServiceCollection services, IConfiguration configuration)
    {
        services.AddOptions<MongodbOptions>()
            .Bind(configuration.GetSection(nameof(MongodbOptions)))
            .ValidateDataAnnotations();
    }
}