#include "annoyrust.h"
#include "kissrandom.h"

typedef ::AnnoyIndexInterface<int32_t, float> *annoy_ptr_t;

rust_annoy_index_t rust_annoy_index_angular_init(int f)
{
    rust_annoy_index_t ptr = new ::AnnoyIndex<int32_t, float, ::Angular, ::Kiss64Random>(f);
    return ptr;
}
rust_annoy_index_t rust_annoy_index_euclidian_init(int f)
{
    rust_annoy_index_t ptr = new ::AnnoyIndex<int32_t, float, ::Euclidean, ::Kiss64Random>(f);
    return ptr;
}
rust_annoy_index_t rust_annoy_index_manhattan_init(int f)
{
    rust_annoy_index_t ptr = new ::AnnoyIndex<int32_t, float, ::Manhattan, ::Kiss64Random>(f);
    return ptr;
}
struct f_vector
{
    vector<float> *vec;
};

struct i_vector
{
    vector<int32_t> *vec;
};

f_vector *f_vector_init()
{
    f_vector *f = new f_vector();
    f->vec = new vector<float>();
    return f;
}

void f_vector_destroy(f_vector *vec)
{
    delete vec->vec;
    delete vec;
}

i_vector *i_vector_init()
{
    i_vector *i = new i_vector();
    i->vec = new vector<int32_t>();
    return i;
}

void i_vector_destroy(i_vector *vec)
{
    delete vec->vec;
    delete vec;
}

annoy_ptr_t cast(rust_annoy_index_t self)
{
    annoy_ptr_t typed_ptr = static_cast<annoy_ptr_t>(self);
    return typed_ptr;
}

void rust_annoy_index_destroy(rust_annoy_index_t self)
{
    annoy_ptr_t typed_ptr = cast(self);
    delete typed_ptr;
}

void rust_annoy_index_add_item(rust_annoy_index_t self, int item, const float *w)
{
    annoy_ptr_t typed_ptr = cast(self);
    typed_ptr->add_item(item, w);
}

void rust_annoy_index_build(rust_annoy_index_t self, int q)
{
    annoy_ptr_t typed_ptr = cast(self);
    typed_ptr->build(q);
}

bool rust_annoy_index_save1(rust_annoy_index_t self, const char *filename)
{
    annoy_ptr_t typed_ptr = cast(self);
    return typed_ptr->save(filename);
}

bool rust_annoy_index_save2(rust_annoy_index_t self, const char *filename, bool prefault)
{
    annoy_ptr_t typed_ptr = cast(self);
    return typed_ptr->save(filename, prefault);
}

void rust_annoy_index_unload(rust_annoy_index_t self)
{
    annoy_ptr_t typed_ptr = cast(self);
    typed_ptr->unload();
}

bool rust_annoy_index_load1(rust_annoy_index_t self, const char *filename)
{
    annoy_ptr_t typed_ptr = cast(self);
    return typed_ptr->load(filename);
}
bool rust_annoy_index_load2(rust_annoy_index_t self, const char *filename, bool prefault)
{
    annoy_ptr_t typed_ptr = cast(self);
    return typed_ptr->load(filename, prefault);
}

float rust_annoy_index_get_distance(rust_annoy_index_t self, int i, int j)
{
    annoy_ptr_t typed_ptr = cast(self);
    return typed_ptr->get_distance(i, j);
}

int rust_annoy_index_get_n_item(rust_annoy_index_t self)
{
    annoy_ptr_t typed_ptr = cast(self);
    return typed_ptr->get_n_items();
}

void rust_annoy_index_verbose(rust_annoy_index_t self, bool v)
{
    annoy_ptr_t typed_ptr = cast(self);
    typed_ptr->verbose(v);
}

void rust_annoy_index_get_nns_by_item(rust_annoy_index_t self, int item, int n, int search_k, i_vector *result, f_vector *distances)
{
    annoy_ptr_t typed_ptr = cast(self);

    typed_ptr->get_nns_by_item(item, n, search_k, result->vec, distances->vec);
}
void rust_annoy_index_get_nns_by_vector(rust_annoy_index_t self, const float *w, int n, int search_k, i_vector *result, f_vector *distances)
{
    annoy_ptr_t typed_ptr = cast(self);
    typed_ptr->get_nns_by_vector(w, n, search_k, result->vec, distances->vec);
}
void rust_annoy_index_get_item(rust_annoy_index_t self, int item, float *v)
{
    annoy_ptr_t typed_ptr = cast(self);
    typed_ptr->get_item(item, v);
}
