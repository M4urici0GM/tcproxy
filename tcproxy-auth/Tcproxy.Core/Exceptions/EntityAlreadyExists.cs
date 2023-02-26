namespace Tcproxy.Core.Exceptions;

public class EntityAlreadyExists<T> : Exception
{
    private new const string Message = "An entity of type {0} ({1}={2}) already exists";

    public EntityAlreadyExists(string key, string value)
        : base(string.Format(Message, typeof(T).Name, key, value))
    {
        
    }
}