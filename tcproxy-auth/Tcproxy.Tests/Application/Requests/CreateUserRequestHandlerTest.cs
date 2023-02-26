using FluentAssertions;
using Moq;
using Tcproxy.Application.Requests.CreateUser;
using Tcproxy.Core.Entities;
using Tcproxy.Core.Exceptions;
using Tcproxy.Persistence.Repositories;
using Xunit;

namespace Tcproxy.Tests.Application.Requests;

public class CreateUserRequestHandlerTest
{
    private readonly CreateUserRequestHandler _sut;
    private readonly Mock<IUserRepository> _userRepositoryMock;


    public CreateUserRequestHandlerTest()
    {
        _userRepositoryMock = new Mock<IUserRepository>();

        _sut = new CreateUserRequestHandler(_userRepositoryMock.Object);
    }

    [Fact(DisplayName = "Should throw EntityAlreadyExists when an user exists with same email")]
    public async Task ShouldThrowEntityAlreadyExists_WhenAnUserExistsWithSameEmail()
    {
        // Arrange
        _userRepositoryMock.Setup(x => x.UserExistsByEmailAsync(
                It.IsAny<string>(),
                It.IsAny<CancellationToken>()))
            .ReturnsAsync(true);

        var request = new CreateUserRequest
        {
            Email = "some_email@some_provider.com",
            Name = "Julia Gates",
            Password = "blueScreen#666",
        };

        // Assert
        await Assert.ThrowsAsync<EntityAlreadyExists<User>>(() => _sut.Handle(request, CancellationToken.None));
    }

    [Fact(DisplayName = "Should create user correctly")]
    public async Task ShouldCreateUserCorrectly()
    {
        // Arrange
        var request = new CreateUserRequest
        {
            Email = "some_email@some_provider.com",
            Name = "Julia Gates",
            Password = "blueScreen#666",
        };

        _userRepositoryMock.Setup(x => x.UserExistsByEmailAsync(
                It.IsAny<string>(),
                It.IsAny<CancellationToken>()))
            .ReturnsAsync(false);

        _userRepositoryMock.Setup(x => x.InsertOneAsync(
                It.IsAny<User>(),
                It.IsAny<CancellationToken>()))
            .ReturnsAsync(new User()
            {
                Name = request.Name,
                Email = request.Email,
                PasswordHash = "SOME_PASSWORD_HASH",
                Active = true,
                Confirmed = false,
            });

        // Act
        var response = await _sut.Handle(request, CancellationToken.None);

        // Assert
        response.Email.Should().Be(request.Email);
        response.Name.Should().Be(request.Name);
    }
}