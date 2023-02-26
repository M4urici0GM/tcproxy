using MongoDB.Driver;
using MongoDB.Driver.Linq;
using Tcproxy.Core.Entities;
using Tcproxy.Persistence.Context;

namespace Tcproxy.Persistence.Repositories;

public interface IUserRepository
{
    /// <summary>
    /// Inserts new user into the database.
    /// </summary>
    /// <param name="user"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    Task<User> InsertOneAsync(User user, CancellationToken cancellationToken);

    /// <summary>
    /// Checks whether a user exists by given email.
    /// </summary>
    /// <param name="email"></param>
    /// <param name="cancellationToken"></param>
    /// <returns></returns>
    Task<bool> UserExistsByEmailAsync(string email, CancellationToken cancellationToken);
}

public class UserRepository : IUserRepository
{
    private readonly IMongoCollection<User> _userCollection;

    public UserRepository(IMongodbContext mongodbContext)
    {
        _userCollection = mongodbContext.GetCollection<User>("users");
    }

    /// <inheritdoc />
    public async Task<User> InsertOneAsync(User user, CancellationToken cancellationToken)
    {
        if (user is null)
        {
            throw new ArgumentNullException(nameof(user));
        }
        
        await _userCollection.InsertOneAsync(user, new InsertOneOptions(), cancellationToken);
        return user;
    }

    /// <inheritdoc />
    public Task<bool> UserExistsByEmailAsync(string email, CancellationToken cancellationToken)
    {
        if (email is null)
        {
            throw new ArgumentNullException(nameof(email));
        }
        
        return _userCollection.AsQueryable()
            .Where(x => x.Email == email)
            .AnyAsync(cancellationToken);
    }
}