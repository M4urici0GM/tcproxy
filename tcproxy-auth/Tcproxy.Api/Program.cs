using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.AspNetCore.Builder;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using tcproxy.api.Application.Extensions;
using tcproxy.api.Middlewares;

var builder = WebApplication.CreateBuilder(args);

builder.Configuration.SetBasePath(builder.Environment.ContentRootPath);
builder.Configuration.AddJsonFile("appsettings.json");
builder.Configuration.AddJsonFile($"appsettings.{builder.Environment.EnvironmentName}.json");
builder.Configuration.AddEnvironmentVariables();

builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();
builder.Services.AddEssentialServices();
builder.Services.AddAppOptions(builder.Configuration);
builder.Services.AddValidators();
builder.Services.AddPersistence();
builder.Services
    .AddControllers()
    .AddJsonOptions(options =>
    {
        options.JsonSerializerOptions.DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull;
        options.JsonSerializerOptions.PropertyNamingPolicy = JsonNamingPolicy.CamelCase;
    });

var app = builder.Build();

app.UseMiddleware<ErrorHandlerMiddleware>();
app.UseSwagger();
app.UseSwaggerUI();
//app.UseHttpsRedirection();
app.UseAuthorization();
app.MapControllers();
app.Run();