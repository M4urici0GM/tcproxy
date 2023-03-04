using FluentAssertions;
using Moq;
using Tcproxy.Application.Requests.AuthenticationChallenge;
using tcproxy.core;
using Tcproxy.Core.Entities;
using Tcproxy.Persistence.Repositories;
using Tcproxy.Tests.Utils;
using Xunit;

namespace Tcproxy.Tests.Application.Requests;

public class AuthenticationChallengeRequestHandlerTest
{
    private readonly AuthenticationChallengeRequestHandler _sut;
    private readonly Mock<IUserRepository> _userRepositoryMock;

    public AuthenticationChallengeRequestHandlerTest()
    {
        _userRepositoryMock = new Mock<IUserRepository>(MockBehavior.Strict);
        _sut = new AuthenticationChallengeRequestHandler(_userRepositoryMock.Object);
    }

    [Fact(DisplayName = "Should return same email as request when user is not found.")]
    public async Task ShouldReturnSameEmailAsRequest_WhenUserDoesntExist()
    {
        // Arrange
        var request = new AuthenticationChallengeRequest("julia@gates.com");
        _userRepositoryMock
            .Setup(x => x.FindByEmailAsync(It.Is<string>(y => y == request.Email), It.IsAny<CancellationToken>()))
            .ReturnsAsync(Option<User>.From(null));

        // Act
        var response = await _sut.Handle(request, CancellationToken.None);

        // Assert
        response.ProfilePicture.Should().BeNull();
        response.UserEmail.Should().Be(request.Email);
    }

    [Fact(DisplayName = "Should return correct user name and email when user does exist")]
    public async Task ShouldReturnCorrectUserNameAndEmail_WhenUserDoesExist()
    {
        // Arrange
        var mockedUser = FakeIt.FakeSingle<User>();
        var request = new AuthenticationChallengeRequest(mockedUser.Email);
         _userRepositoryMock
            .Setup(x => x.FindByEmailAsync(It.Is<string>(y => y == mockedUser.Email), It.IsAny<CancellationToken>()))
            .ReturnsAsync(Option<User>.From(mockedUser));
         
        // Act
        var response = await _sut.Handle(request, CancellationToken.None);
        
        // Assert
        response.Should().NotBeNull();
        response.UserEmail.Should().Be(mockedUser.Email);
        response.UserName.Should().Be(mockedUser.Name);
    }
}