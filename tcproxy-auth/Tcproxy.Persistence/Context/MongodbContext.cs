using Microsoft.Extensions.Options;
using MongoDB.Bson.Serialization.Conventions;
using MongoDB.Driver;
using Tcproxy.Core.Options;

namespace Tcproxy.Persistence.Context;

public interface IMongodbContext
{
    IMongoCollection<T> GetCollection<T>(string collectionName);
}

public class MongodbContext : IMongodbContext
{
    private readonly IMongoDatabase _mongoDatabase;
    
    public MongodbContext(IOptions<MongodbOptions> options)
    {
        var conventionPack = new  ConventionPack() {new CamelCaseElementNameConvention()};
        ConventionRegistry.Register("camelCase", conventionPack, t => true);
        
        var mongodbClient = new MongoClient(options.Value.ConnectionString);
        _mongoDatabase = mongodbClient.GetDatabase(options.Value.DatabaseName);
    }
    
    public IMongoCollection<T> GetCollection<T>(string collectionName)
    {
        return _mongoDatabase.GetCollection<T>(collectionName);
    }
}