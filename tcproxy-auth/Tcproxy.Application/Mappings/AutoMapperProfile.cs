using AutoMapper;
using Tcproxy.Application.Responses;
using Tcproxy.Core.Entities;

namespace Tcproxy.Application.Mappings;

public class AutoMapperProfile : Profile
{
    public AutoMapperProfile()
    {
        CreateMap<User, UserResponse>();
    }
}