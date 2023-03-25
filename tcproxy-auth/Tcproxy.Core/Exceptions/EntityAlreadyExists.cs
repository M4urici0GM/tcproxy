namespace Tcproxy.Core.Exceptions;

public class EntityAlreadyExists : Exception
{
    private new const string Message = "An entity of type {0} ({1}={2}) already exists";

    public EntityAlreadyExists(string entityName, string key, string value)
        : base(string.Format(Message, entityName, key, value))
    {
        
    }
}