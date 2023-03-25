using FluentValidation;

namespace Tcproxy.Application.Requests.CreateUser;

/// <summary>
/// Validation for CreateUserRequest
/// </summary>
public class CreateUserRequestValidator : AbstractValidator<CreateUserRequest>
{
    public CreateUserRequestValidator()
    {
        RuleFor(x => x.Email)
            .NotEmpty()
            .WithMessage("Email address is required")
            .EmailAddress()
            .OverridePropertyName("email");

        RuleFor(x => x.FirstName)
            .NotEmpty()
            .WithMessage("Name is required")
            .OverridePropertyName("firstName");
        
        RuleFor(x => x.LastName)
            .NotEmpty()
            .WithMessage("Last name is required")
            .OverridePropertyName("lastName");
        
        RuleFor(x => x.Password)
            .NotEmpty()
            .OverridePropertyName("password")
            .WithMessage("Password is required")
            .Matches(@"^(?=.*[A-Za-z])(?=.*\d)(?=.*[@$!%*#?&])[A-Za-z\d@$!%*#?&]{6,30}$")
            .WithMessage(@"Password must have at least 6 and maximum of 30 characters,at least one number and one special character");
    }
}