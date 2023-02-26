namespace Tcproxy.Core;

/// <summary>
/// Represents something that can be null or not.
/// Note that nullable is disable across all DLLs,
/// so having a shorthand for detecting nulls without using nulls directly
/// is a good practice.
/// </summary>
/// <typeparam name="T"></typeparam>
public class Option<T>
{
    private readonly T _item;

    private Option()
    {
        
    }

    private Option(T item)
    {
        _item = item;
    }

    public static Option<T> From(T item)
    {
        return (item == null)
            ? new Option<T>()
            : new Option<T>(item);
    }

    public bool IsSome()
    {
        return _item != null;
    }
    
    public bool IsNone()
    {
        return _item == null;
    }

    public T Unwrap()
    {
        if (_item == null)
            throw new ArgumentNullException("item");

        return _item;
    }
}