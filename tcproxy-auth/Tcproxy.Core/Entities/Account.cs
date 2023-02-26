namespace Tcproxy.Core.Entities;

[Serializable]
public class Account : BaseEntity
{
    public string AccountName { get; set; } = string.Empty;
    public Guid UserId { get; set; }
}