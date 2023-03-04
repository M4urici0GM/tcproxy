using MongoDB.Bson.Serialization.Attributes;

namespace Tcproxy.Core.Entities;

[Serializable]
[BsonIgnoreExtraElements]
public class User : BaseEntity
{
    public string Name { get; set; } = string.Empty;
    public string Email { get; set; } = string.Empty;
    public string PasswordHash { get; set; } = string.Empty;
    public bool Confirmed { get; set; } = false;
    public bool Active { get; set; } = false;
    public string ProfilePicture { get; set; } = string.Empty;
}